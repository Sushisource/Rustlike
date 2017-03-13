extern crate nalgebra as na;
extern crate rand;

use super::super::util::Point;

use self::na::{Vector2};
use self::rand::{thread_rng, Rng};

pub struct Room {
  pub top_left: Point,
  pub bottom_right: Point
}

impl Room {
  pub fn new(center: Point, width: f32, height: f32) -> Room {
    let scale = Vector2::new(width / 2.0, height / 2.0);
    Room { top_left: center - scale, bottom_right: center + scale }
  }

  /// Creates a new `room` randomly placed somewhere in unit space
  pub fn new_rand() -> Room {
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(-1.0, 1.0);
    let c_y: f32 = rng.gen_range(-1.0, 1.0);
    Room::new(Point::new(c_x, c_y),
              rng.gen_range(0.0, 0.5), rng.gen_range(0.0, 0.5))
  }
}