extern crate ggez;
extern crate geo;

pub mod level_renderer;
pub mod direction;
mod rooms;
mod ca_simulator;

use self::geo::{MultiPoint};
use self::geo::algorithm::boundingbox::BoundingBox;

use super::util::Meters;
use self::rooms::Room;
use self::ca_simulator::CASim;
// TODO: Dedupe inside casim
const CA_W: usize = 266;
const CA_H: usize = 150;

type CellGrid = [[bool; CA_H]; CA_W];
type Point = self::geo::Point<f32>;

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave_sim: CASim,
  pub level_gen_finished: bool,
  pub rooms: Vec<Room>,
  gen_stage: u8,
  width: Meters,
  height: Meters,
}

impl Level {
  pub fn new() -> Level {
    Level {
      cave_sim: CASim::new(),
      level_gen_finished: false,
      rooms: Vec::new(),
      gen_stage: 0,
      width: 177.7,
      height: 100.0,
    }
  }

  pub fn tick_level_gen(&mut self) -> () {
    let stage_complete = match self.gen_stage {
      0 => self.tick_cavesim(),
      1 => self.tick_roomsim(),
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
    let cavebf: Vec<Point> = self.cave_sim.ca_boundary.iter()
      .map(|&(x, y)| self.uspace_to_wspace(ca_to_uspace(x,y))).collect();
    let caveb: MultiPoint<_> = cavebf.into();
    let cave_bb = caveb.bbox().unwrap();
    if self.rooms.len() < 20 {
      loop {
        let room = Room::new_rand((0.0, self.width), (0.0, self.height));
        let avoids_other_rooms =
          self.rooms.iter().all(|ref r| !room.intersects(r));
        let in_cave = room.center.x < cave_bb.xmax && room.center.x > cave_bb.xmin
                      && room.center.y < cave_bb.ymax && room.center.y > cave_bb.ymin;
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

  fn place_obstacles(&mut self) -> bool {
    // Grow some ponds using our CA generation method
    true
  }

  /// Converts world space to unit space
  fn wspace_to_uspace(&self, p: Point) -> Point {
    Point::new(p.x() / self.width, p.y() / self.height)
  }

  /// Converts unit space to world space
  fn uspace_to_wspace(&self, p: Point) -> Point {
    Point::new(p.x() * self.width, p.y() * self.height)
  }

  fn middle(&self) -> Point { Point::new(self.width / 2.0, self.height / 2.0) }
}

/// Converts cellular automata space to unit space
fn ca_to_uspace(x: i32, y: i32) -> Point {
  let xp = (x as f32) / (CA_W as f32);
  let yp = (y as f32) / (CA_H as f32);
  Point::new(xp, yp)
}
