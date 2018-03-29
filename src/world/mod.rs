extern crate ggez;
extern crate nalgebra as na;
extern crate ncollide as nc;

use agents::player::Player;
use dungeongen::Level;
use util::drawablept::DrawablePt;
use super::util::Point;

pub mod render;

// TODO: Second param is wrong, represents defined-by-me data. Need
// to have some kind of ID system for all entities
pub type CollW = nc::world::CollisionWorld2<f32, f32>;

/// The entire world. Contains all world objects, and handles interaction
/// between subsystems.
pub struct World {
  level: Level,
  player: Player,
  collision: CollW,
}

impl World {
  pub fn new() -> World {
    let level = Level::new();
    let player = Player::new(DrawablePt(level.middle()).into());
    World {
      level: level.into(),
      player,
      collision: CollW::new(0.02),
    }
  }

  fn add_level_contents_to_collision(&mut self) -> () {
    self.level.populate_collision_world(&mut self.collision);
    self.collision.update();
  }

  fn collision_test(&self, p: &Point) -> () {
    let mut cgs = nc::world::CollisionGroups::new();
    cgs.set_membership(&[2]);
    cgs.set_whitelist(&[1]);
    let collisions = self.collision.interferences_with_point(p, &cgs);
    // TODO: Something useful
    // for c in collisions {
    //   println!("{}", c.position());
    // }
  }
}
