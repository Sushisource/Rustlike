extern crate ggez;
extern crate nalgebra as na;
extern crate rand;

use self::rand::{thread_rng, Rng};
use self::rand::distributions::{IndependentSample, Normal};
use self::ggez::{Context, GameError};
use self::ggez::graphics::{rectangle, DrawMode, DrawParam, Drawable, Point,
                           Rect};

use super::super::util::Meters;

#[derive(Debug)]
pub struct Room {
  pub center: Point,
  pub width: f32,
  pub height: f32,
}

impl Room {
  pub fn new(center: Point, width: Meters, height: Meters) -> Room {
    Room { center: center,
           width: width,
           height: height, }
  }

  /// Creates a new `room` randomly placed somewhere in the provided range
  pub fn new_rand((x_min, x_max): (Meters, Meters),
                  (y_min, y_max): (Meters, Meters))
                  -> Room {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    let scaler = (x_max - x_min).min(y_max - y_min) as f64;
    let sizer = Normal::new(scaler / 10.0, scaler / 20.0);
    let mut get_siz = || {
      sizer.ind_sample(&mut rng).abs().max(scaler / 30.0).min(scaler / 2.0) as
      f32
    };
    Room::new(Point::new(c_x, c_y), get_siz(), get_siz())
  }

  /// Tests intersection with another room. Returns true if they intersect.
  pub fn intersects(&self, other: &Room) -> bool {
    let r1: Rect = self.into();
    let r2: Rect = other.into();
    !(r2.left() > r1.right() || r2.right() < r1.left() ||
      r2.top() < r1.bottom() || r2.bottom() > r1.top())
  }
}

impl Drawable for Room {
  fn draw_ex(&self, ctx: &mut Context, param: DrawParam)
             -> Result<(), GameError> {
    let mut r: Rect = self.into();
    r.x = param.dest.x;
    r.y = param.dest.y;
    r.w *= param.scale.x;
    r.h *= param.scale.y;
    rectangle(ctx, DrawMode::Fill, r)
  }
}

impl<'a> From<&'a Room> for Rect {
  fn from(r: &Room) -> Rect {
    Rect { x: r.center.x,
           y: r.center.y,
           w: r.width,
           h: r.height, }
  }
}
