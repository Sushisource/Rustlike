extern crate nalgebra as na;
extern crate rand;

use self::na::{Vector2};
use self::rand::{thread_rng, Rng};
use self::rand::distributions::{Normal, IndependentSample};

use super::super::util::Point;

pub struct Room {
  pub top_left: Point,
  pub bottom_right: Point
}

impl Room {
  pub fn new(center: Point, width: f32, height: f32) -> Room {
    assert!(width >= 0.0);
    assert!(height >= 0.0);
    let scale = Vector2::new(width / 2.0, height / 2.0);
    Room { top_left: center - scale, bottom_right: center + scale }
  }

  /// Creates a new `room` randomly placed somewhere in unit space
  pub fn new_rand() -> Room {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(-1.0, 1.0);
    let c_y: f32 = rng.gen_range(-1.0, 1.0);
    let sizer = Normal::new(0.1, 0.2);
    let mut get_siz = || {
      ((sizer.ind_sample(&mut rng) as f32).abs()).max(0.05).min(0.3)
    };
    Room::new(Point::new(c_x, c_y), get_siz(), get_siz())
  }

  /// Tests intersection with another room. Returns true if they intersect.
  pub fn intersects(&self, other: &Room) -> bool {
    return !(other.top_left.x > self.bottom_right.x ||
      other.bottom_right.x < self.top_left.x ||
      other.top_left.y > self.bottom_right.y ||
      other.bottom_right.y < self.top_left.y)
  }
}