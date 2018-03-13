// TODO: I hate this. Get rid of this.

extern crate ggez;
extern crate geo;

use std;

use util;
use self::ggez::graphics::Point2;
use self::geo::algorithm::map_coords::MapCoords;

type LevelPoint = util::Point;

// Sorta lame that we have to do this b/c can't implement traits for non-crate
// types
#[derive(Copy, Clone, Debug)]
pub struct DrawablePt(pub LevelPoint);

impl From<DrawablePt> for Point2 {
  fn from(dp: DrawablePt) -> Self {
    let DrawablePt(p) = dp;
    Point2::new(p.x(), p.y())
  }
}

impl From<Point2> for DrawablePt {
  fn from(dp: Point2) -> Self {
    DrawablePt(LevelPoint::new(dp.x, dp.y))
  }
}

impl std::ops::Mul for DrawablePt {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    let DrawablePt(p) = self;
    let DrawablePt(p2) = rhs;
    DrawablePt(LevelPoint::new(p.x() * p2.x(), p.y() * p2.y()))
  }
}

impl std::ops::Mul<Point2> for DrawablePt {
  type Output = Self;

  fn mul(self, rhs: Point2) -> Self {
    let DrawablePt(p) = self;
    DrawablePt(LevelPoint::new(p.x() * rhs.coords.x, p.y() * rhs.coords.y))
  }
}

impl DrawablePt {
  /// Truncates floating point numbers to avoid rendering artifacts
  pub fn snap(&self) -> Self {
    let DrawablePt(p) = *self;
    DrawablePt(p.map_coords(&|&(x, y)| (x.ceil(), y.ceil())))
  }
}
