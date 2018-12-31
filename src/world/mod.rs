extern crate ncollide2d as nc;

use crate::agents::player::Player;
use crate::collision::{new_collw, CollW, CollidableDat, GameObjRegistrar};
use crate::dungeongen::level::Level;
use crate::util::Point;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod render;

/// The entire world. Contains all world objects, and handles interaction
/// between subsystems.
pub struct World {
  level: Level,
  player: Player,
  collision: CollW,
  // TODO: Move to Specs and use that for entity IDs?
  next_eid: AtomicUsize, // Could be atomic
}

impl World {
  pub fn new() -> World {
    let level = Level::new();
    let player = Player::new(level.middle());
    World { level, player, collision: new_collw(), next_eid: AtomicUsize::new(0) }
  }

  fn add_level_contents_to_collision(&mut self) -> () {
    let rooms = self.level.produce_collidables();
    for r in rooms {
      let cw_dat = CollidableDat::new(r.coltype(), self.next_eid.fetch_add(1, Ordering::Relaxed));
      self.collision.register(r, cw_dat);
    }
    self.collision.update();
  }

  fn collision_test(&self, p: Point) -> () {
    let mut cgs = nc::world::CollisionGroups::new();
    cgs.set_membership(&[2]);
    cgs.set_whitelist(&[1]);

    let collisions = self.collision.interferences_with_point(&p, &cgs);
    for c in collisions {
      println!("{}", c.position());
    }
  }
}
