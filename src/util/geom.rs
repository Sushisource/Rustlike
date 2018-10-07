use collision::{Collidable, CollidableType, CollisionRect, Shape2D};
use ggez::graphics::Rect;
use na;
use na::{Isometry2, Vector2};
use nc::query;
use nc::shape::{Compound, ShapeHandle};
use nc::world::CollisionGroups;
use std::f32::consts::PI;
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

impl<'a> Collidable for &'a CenterOriginRect {
  fn location(&self) -> Isometry2<Meters> {
    Isometry2::new(self.center().coords, na::zero())
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
    self.center
  }
  fn width(&self) -> f32 {
    self.width
  }
  fn height(&self) -> f32 {
    self.height
  }
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
pub type IntPoint = na::Point2<i32>;

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

/// Given some existing rooms (clustered around the origin) and a new room (at the origin),
/// move the new room away from the origin at the exit angle until flush with the edge of one
/// of the existing rooms. Rooms can't be bigger than 1000 meters in any direction.
pub fn snap_to_existing_rooms(
  rooms: &Vec<GridRect>,
  new_room: GridRect,
  exit_angle: f32,
) -> GridRect {
  let walk_vec: Vector2<Meters> = PolarVec::new(1000.0, exit_angle).into();
  let walk_to_pt = IntPoint::new(walk_vec.x as i32, walk_vec.y as i32);
  let walk_list = walk_grid(IntPoint::new(0, 0), walk_to_pt);
  let orig_w = new_room.width;
  let orig_h = new_room.height;
  let nr_coll = &new_room as &CenterOriginRect;
  let nr_shape1 = nr_coll.shape();
  let nr_shape2: &CollisionRect = nr_shape1.as_shape().unwrap();
  let nr_shape_half_w = nr_shape2.half_extents().x; // - 0.2;
  let nr_shape_half_h = nr_shape2.half_extents().y; // - 0.2;
  let nr_shape: CollisionRect = CollisionRect::new(Vector2::new(nr_shape_half_w, nr_shape_half_h));
  let coll_rooms: Vec<&CenterOriginRect> = rooms.iter().map(|r| r as &CenterOriginRect).collect();
  let compound_room = compoundify(&coll_rooms);
  let mut last_pt = nr_coll.location();
  for walkpt in walk_list {
    let cur_pt = Isometry2::new(
      Vector2::new(walkpt.x as f32 + nr_shape_half_w, walkpt.y as f32 + nr_shape_half_h),
      0.0,
    );
    let contact = query::contact(&origin(), &compound_room, &cur_pt, &nr_shape, 0.0);
    let is_contact = contact.is_some();

    if !is_contact {
      break;
    }
    // Same as cur pt without the center shift
    last_pt = Isometry2::new(Vector2::new(walkpt.x as f32, walkpt.y as f32), 0.0);
  }
  let intified: Vector2<i32> =
    Vector2::new(last_pt.translation.vector.x as i32, last_pt.translation.vector.y as i32);
  GridRect::new(orig_w, orig_h, IntPoint::from_coordinates(intified))
}

fn compoundify(shapes: &Vec<impl Collidable>) -> Compound<f32> {
  Compound::new(shapes.iter().map(|r| (r.location(), r.shape())).collect())
}

fn walk_grid(p1: IntPoint, p2: IntPoint) -> Vec<IntPoint> {
  // Thanks RedBlob games
  let dx = p2.x - p1.x;
  let dy = p2.y - p1.y;
  let nx = dx.abs() as f32;
  let ny = dy.abs() as f32;
  let sign_x = if dx > 0 { 1 } else { -1 };
  let sign_y = if dy > 0 { 1 } else { -1 };
  let mut p = p1.clone();
  let mut points = vec![p.clone()];
  let (mut ix, mut iy) = (0.0, 0.0);
  while ix < nx || iy < ny {
    if (0.5 + ix) / nx == (0.5 + iy) / ny {
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
    points.push(p.clone())
  }
  points
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_simple_snap() {
    let existing = vec![GridRect::new(1, 1, IntPoint::new(0, 0))];
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, 0.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(1, 0)), moved);
  }

  #[test]
  fn test_series_of_snaps() {
    let mut existing = vec![GridRect::new(1, 1, IntPoint::new(0, 0))];
    // Up
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, PI / 2.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(0, 1)), moved);
    existing.push(moved);
    // Right
    let new = GridRect::new(4, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, 0.0);
    assert_eq!(GridRect::new(4, 1, IntPoint::new(1, 0)), moved);
    existing.push(moved);
    // Up again, two more times
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, PI / 2.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(0, 2)), moved);
    existing.push(moved);
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, PI / 2.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(0, 3)), moved);
    existing.push(moved);
    // Diagonally up and right
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, PI / 4.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(1, 1)), moved);
    existing.push(moved);
    // Right again
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    let moved = snap_to_existing_rooms(&existing, new, 0.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(5, 0)), moved);
    existing.push(moved);
  }
}
