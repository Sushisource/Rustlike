use collision::{Collidable, CollidableType, CollisionRect, Shape2D};
use ggez::graphics::Rect;
use na;
use na::{Vector2, Isometry2};
use nc::shape::ShapeHandle;
use nc::world::CollisionGroups;
use util::{Meters, Point};

pub trait CenterOriginRect {
  fn center(&self) -> Point;
  fn width(&self) -> Meters;
  fn height(&self) -> Meters;
  fn left_edge(&self) -> Meters {
    self.center().x - self.width() / 2.0
  }
  fn right_edge(&self) -> Meters {
    self.center().x + self.width() / 2.0
  }
  fn top_edge(&self) -> Meters {
    self.center().y - self.height() / 2.0
  }
  fn bottom_edge(&self) -> Meters {
    self.center().y + self.height() / 2.0
  }
}

impl Collidable for CenterOriginRect {
  fn location(&self) -> Isometry2<Meters> { Isometry2::new(self.center().coords, na::zero()) }
  fn shape(&self) -> Shape2D {
    ShapeHandle::new(CollisionRect::new(Vector2::new(self.width() / 2.0, self.height() / 2.0)))
  }
  fn collision_group(&self) -> CollisionGroups { CollisionGroups::new() }
  fn coltype(&self) -> CollidableType { CollidableType::Generic }
}

#[derive(new, Debug, PartialEq, Copy, Clone)]
pub struct CenteredRect {
  pub center: Point,
  pub width: Meters,
  pub height: Meters,
}

impl CenterOriginRect for CenteredRect {
  fn center(&self) -> Point { self.center }
  fn width(&self) -> f32 { self.width }
  fn height(&self) -> f32 { self.height }
}

impl<'a> Into<Rect> for &'a CenterOriginRect {
  fn into(self) -> Rect {
    Rect {
      // GGEZ rect is top-left origin
      x: self.center().x - self.width() / 2.0,
      y: self.center().y - self.height() / 2.0,
      w: self.width(),
      h: self.height(),
    }
  }
}

pub fn origin() -> Isometry2<Meters> { Isometry2::new(Vector2::zeros(), 0.0) }

#[derive(new, Debug, PartialEq, Copy, Clone)]
pub struct PolarVec {
  pub distance: Meters,
  pub angle: f32
}

impl Into<Vector2<Meters>> for PolarVec {
  fn into(self) -> Vector2<Meters> {
    Vector2::new(self.distance * self.angle.cos(), self.distance * self.angle.sin())
  }
}