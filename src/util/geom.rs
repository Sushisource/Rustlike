use collision::{Collidable, CollidableType, CollisionRect, Shape2D};
use ggez::graphics::Rect;
use na::Vector2;
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
  fn location(&self) -> Point { self.center() }
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
