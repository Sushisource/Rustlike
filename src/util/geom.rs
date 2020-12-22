use crate::{
  collision::{Collidable, CollidableType, CollisionRect, Shape2D},
  dungeongen::{direction::Direction, level::Wall, level::WALL_THICKNESS},
  util::{Meters, Point},
};
use nalgebra::{Isometry2, Vector2};
use ncollide2d::{shape::ShapeHandle, world::CollisionGroups};
use num::zero;

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

impl<'a> CenterOriginRect + 'a {
  /// Generates walls for the rect. Walls are `WALL_THICKNESS` thick
  pub fn gen_walls(&self) -> Vec<(Wall, Direction)> {
    let mut retme = vec![];
    for d in Direction::compass() {
      let d = *d;
      let full_w = self.width() + WALL_THICKNESS;
      let full_h = self.height() + WALL_THICKNESS;
      match d {
        Direction::North | Direction::South => {
          let yoffset = self.center().y + self.height() / 2.0 * d.to_tup().1;
          let wall_c = Point::new(self.center().x, yoffset);
          retme.push((Wall::new(wall_c, full_w, WALL_THICKNESS), d));
        }
        _ => {
          let xoffset = self.center().x + self.width() / 2.0 * d.to_tup().0;
          let wall_c = Point::new(xoffset, self.center().y);
          retme.push((Wall::new(wall_c, WALL_THICKNESS, full_h), d));
        }
      }
    }
    retme
  }
}

impl<'a> Collidable for &'a CenterOriginRect {
  fn location(&self) -> Isometry2<Meters> {
    Isometry2::new(self.center().coords, zero())
  }
  fn shape(&self) -> Shape2D {
    ShapeHandle::new(CollisionRect::new(Vector2::new(self.width() / 2.0, self.height() / 2.0)))
  }
  fn collision_group(&self) -> CollisionGroups {
    CollisionGroups::new()
  }
  fn coltype(&self) -> CollidableType {
    CollidableType::Generic
  }
}

#[derive(new, Debug, PartialEq, Copy, Clone)]
pub struct CenteredRect {
  pub center: Point,
  pub width: Meters,
  pub height: Meters,
}

impl CenterOriginRect for CenteredRect {
  fn center(&self) -> Point {
    self.center.clone()
  }
  fn width(&self) -> f32 {
    self.width
  }
  fn height(&self) -> f32 {
    self.height
  }
}

// impl<'a> Into<Rect> for &'a CenterOriginRect {
//   fn into(self) -> Rect {
//     Rect {
//       // GGEZ rect is top-left origin
//       x: self.center().x - self.width() / 2.0,
//       y: self.center().y - self.height() / 2.0,
//       w: self.width(),
//       h: self.height(),
//     }
//   }
// }

pub fn origin() -> Isometry2<Meters> {
  Isometry2::new(Vector2::zeros(), 0.0)
}

#[derive(new, Debug, PartialEq, Copy, Clone)]
pub struct PolarVec {
  pub distance: Meters,
  pub angle: f32,
}

impl Into<Vector2<Meters>> for PolarVec {
  fn into(self) -> Vector2<Meters> {
    Vector2::new(self.distance * self.angle.cos(), self.distance * self.angle.sin())
  }
}

// Gridded geometry below here ====================================================================
pub type IntPoint = nalgebra::Point2<i32>;

#[derive(new, Debug, PartialEq)]
pub struct GridRect {
  pub width: u32,
  pub height: u32,
  pub top_left: IntPoint,
}

impl CenterOriginRect for GridRect {
  fn center(&self) -> Point {
    Point::new(
      self.top_left.x as f32 + self.width as f32 / 2.0,
      self.top_left.y as f32 + self.height as f32 / 2.0,
    )
  }
  fn width(&self) -> f32 {
    self.width as f32
  }
  fn height(&self) -> f32 {
    self.height as f32
  }
}

pub fn walk_grid(p1: IntPoint, p2: IntPoint) -> Vec<IntPoint> {
  // Thanks RedBlob games
  let dx = p2.x - p1.x;
  let dy = p2.y - p1.y;
  let nx = dx.abs() as f32;
  let ny = dy.abs() as f32;
  let sign_x = if dx > 0 { 1 } else { -1 };
  let sign_y = if dy > 0 { 1 } else { -1 };
  let mut p = Clone::clone(&p1);
  let mut points = vec![Clone::clone(&p)];
  let (mut ix, mut iy) = (0.0, 0.0);
  while ix < nx || iy < ny {
    if ((0.5 + ix) / nx - (0.5 + iy) / ny).abs() < 0.005 {
      // next step is diagonal
      p.x += sign_x;
      p.y += sign_y;
      ix += 1.0;
      iy += 1.0;
    } else if (0.5 + ix) / nx < (0.5 + iy) / ny {
      //next step is horizontal
      p.x += sign_x;
      ix += 1.0;
    } else {
      //next step is vertical
      p.y += sign_y;
      iy += 1.0;
    }
    points.push(Clone::clone(&p))
  }
  points
}
