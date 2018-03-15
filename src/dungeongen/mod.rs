extern crate ggez;
extern crate nalgebra;
extern crate ncollide;

pub mod level_renderer;
pub mod direction;

mod rooms;
mod ca_simulator;
mod blobstacle;

use std::sync::Arc;
use super::util::{Meters, Point};
use self::nalgebra as na;
use self::na::Isometry2;
use self::rooms::Room;
use self::ca_simulator::CASim;
use self::blobstacle::Blobstacle;
use self::nalgebra::Point2;
use self::ncollide::shape::{Polyline, Shape};
use self::ncollide::bounding_volume::AABB;

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave_sim: CASim,
  pub level_gen_finished: bool,
  pub rooms: Vec<Room>,
  pub obstacles: Vec<Blobstacle>,
  gen_stage: u8,
  width: Meters,
  height: Meters,
}

impl Level {
  pub fn new() -> Level {
    Level {
      // TODO: Right now the dimensions of this sim need to have the same ratio
      // as the screen or it gets squished
      cave_sim: CASim::new(266, 150, 1.0),
      level_gen_finished: false,
      rooms: Vec::new(),
      obstacles: Vec::new(),
      gen_stage: 0,
      width: 80.0,
      height: 45.0,
    }
  }

  pub fn tick_level_gen(&mut self) -> () {
    let stage_complete = match self.gen_stage {
      0 => self.tick_cavesim(),
      1 => self.tick_roomsim(),
      2 => self.place_obstacles(),
      // TODO: ensure all rooms are connected after placing obstacles (spanning tree)
      _ => false,
    };
    if stage_complete {
      self.gen_stage += 1
    }
  }

  fn tick_cavesim(&mut self) -> bool {
    self.cave_sim.tick()
  }

  fn tick_roomsim(&mut self) -> bool {
    // Room centers should be within the bounding box of the cave
    let cave_bb = self.cave_bound_box();
    if self.rooms.len() < 20 {
      loop {
        let room = Room::new_rand((0.0, self.width), (0.0, self.height));
        let avoids_other_rooms =
          self.rooms.iter().all(|ref r| !room.intersects(r));
        let in_cave = room.center.x < cave_bb.maxs().x
          && room.center.x > cave_bb.mins().x
          && room.center.y < cave_bb.maxs().y
          && room.center.y > cave_bb.mins().y;
        if avoids_other_rooms && in_cave {
          self.rooms.push(room);
          break;
        }
      }
      false
    } else {
      println!("Done placing rooms");
      true
    }
  }

  pub fn cave_bound_box(&self) -> AABB<Point> {
    let cavebf: Vec<Point> = self.cave_bounds();
    let cave_ixs: Vec<Point2<usize>> = (0..cavebf.len())
      .map(|i| {
        let to = if i + 1 == cavebf.len() { 0 } else { i + 1 };
        Point2::new(i, to)
      })
      .collect();
    let cave_polyline =
      Polyline::new(Arc::new(cavebf), Arc::new(cave_ixs), None, None);
    let cave_pos = na::one::<Isometry2<f32>>();
    cave_polyline.aabb(&cave_pos)
  }

  fn cave_bounds(&self) -> Vec<Point> {
    self
      .cave_sim
      .uspace_boundary(Point::new(0.0, 0.0))
      .iter()
      .map(|&p| self.uspace_to_lspace(p))
      .collect()
  }

  fn place_obstacles(&mut self) -> bool {
    // Grow some ponds using our CA generation method
    let test_pond = Blobstacle::new(Point::new(30.0, 30.0));
    let test_pond2 = Blobstacle::new(Point::new(5.5, 5.1));
    let test_pond3 = Blobstacle::new(Point::new(20.8, 20.8));
    self.obstacles.push(test_pond);
    self.obstacles.push(test_pond2);
    self.obstacles.push(test_pond3);
    true
  }

  /// Converts level space to unit space
  pub fn lspace_to_uspace(&self, p: Point) -> Point {
    Point::new(p.x / self.width, p.y / self.height)
  }

  /// Converts unit space to level space
  pub fn uspace_to_lspace(&self, p: Point) -> Point {
    Point::new(p.x * self.width, p.y * self.height)
  }

  pub fn middle(&self) -> Point {
    Point::new(self.width / 2.0, self.height / 2.0)
  }
}
