extern crate ggez;
extern crate geo;

pub mod level_renderer;
pub mod direction;
mod rooms;
mod ca_simulator;
mod blobstacle;

use self::geo::MultiPoint;
use self::geo::algorithm::boundingbox::BoundingBox;

use super::util::Meters;
use self::rooms::Room;
use self::ca_simulator::CASim;
use self::blobstacle::Blobstacle;

type Point = self::geo::Point<f32>;

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
      cave_sim: CASim::new(1.0),
      level_gen_finished: false,
      rooms: Vec::new(),
      obstacles: Vec::new(),
      gen_stage: 0,
      width: 177.7,
      height: 100.0,
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
    let cavebf: Vec<Point> = self.cave_sim.uspace_boundary(Point::new(0.0, 0.0))
                                 .iter().map(|&p| self.uspace_to_wspace(p))
                                 .collect();
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
    let test_pond = Blobstacle::new(self.middle());
    let test_pond2 = Blobstacle::new(Point::new(5.5, 5.1));
    let test_pond3 = Blobstacle::new(Point::new(20.8, 20.8));
    self.obstacles.push(test_pond);
    self.obstacles.push(test_pond2);
    self.obstacles.push(test_pond3);
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

