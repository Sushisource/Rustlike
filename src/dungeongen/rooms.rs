extern crate nalgebra as na;
extern crate rand;
extern crate ggez;

use self::rand::{thread_rng, Rng};
use self::rand::distributions::{Normal, IndependentSample};
use self::ggez::graphics::{Rect, Point};

use super::super::util::Meters;

#[derive(Debug)]
pub struct Room {
  pub center: Point,
  pub width: f32,
  pub height: f32,
}

impl Room {
  pub fn new(center: Point, width: Meters, height: Meters) -> Room {
    Room {
      center: center,
      width: width,
      height: height,
    }
  }

  /// Creates a new `room` randomly placed somewhere in the provided range
  pub fn new_rand((x_min, x_max): (Meters, Meters),
                  (y_min, y_max): (Meters, Meters))
                  -> Room {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    let sizer = Normal::new(15.0, 20.0);
    let mut get_siz =
      || sizer.ind_sample(&mut rng).abs().max(8.0).min(40.0) as f32;
    Room::new(Point::new(c_x, c_y), get_siz(), get_siz())
  }

  /// Tests intersection with another room. Returns true if they intersect.
  pub fn intersects(&self, other: &Room) -> bool {
    let s: Rect = self.into();
    let o: Rect = other.into();
    o.left() < s.right() && o.right() > s.left() && o.top() > s.bottom() &&
    o.bottom() < s.top()
  }
}

impl<'a> From<&'a Room> for Rect {
  fn from(r: &Room) -> Rect {
    Rect {
      x: r.center.x / 100.0,
      y: r.center.y / 100.0,
      w: r.width / 20.0,
      h: r.height / 20.0,
    }
  }
}
