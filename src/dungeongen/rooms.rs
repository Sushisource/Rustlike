extern crate ggez;
extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate rand;

use self::ggez::{Context, GameResult};
use self::ggez::graphics::{Color, DrawMode, Rect, rectangle, set_color};
use self::na::Vector2;
use self::rand::{Rng, thread_rng};
use self::rand::distributions::{IndependentSample, Normal};
use super::direction::Direction;
use super::super::util::*;

static WALL_THICKNESS: Meters = 0.2;
static DOOR_WIDTH: Meters = 1.1;

trait CenterOriginRect {
  fn center(&self) -> Point;
  fn width(&self) -> Meters;
  fn height(&self) -> Meters;
}

#[derive(Debug)]
pub struct Room {
  pub center: Point,
  pub width: Meters,
  pub height: Meters,
  door: Rect,
  door_side: Direction,
  walls: Vec<Wall>,
}

impl Room {
  pub fn new(center: Point, width: Meters, height: Meters, door: Rect, door_side: Direction)
             -> Room {
    let walls = Room::gen_walls(center, width, height, door, door_side);
    Room { center, width, height, door, door_side, walls }
  }

  /// Creates a new `room` randomly placed somewhere in the provided range
  pub fn new_rand((x_min, x_max): (Meters, Meters), (y_min, y_max): (Meters, Meters)) -> Room {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    let (room_w, room_h) = {
      let scaler = (x_max - x_min).min(y_max - y_min) as f64;
      let sizer = Normal::new(scaler / 10.0, scaler / 20.0);
      let mut get_siz = || {
        sizer
          .ind_sample(&mut rng)
          .abs()
          .max(scaler / 30.0)
          // Rooms must be at least 1m sq so a door can fit, regardless of sizing params, plus
          // a bit extra for wiggle room
          .max((DOOR_WIDTH + 0.2).into())
          .min(scaler / 2.0) as Meters
      };
      (get_siz(), get_siz())
    };
    // Add a door somewhere along the room edge
    let side = rng.choose(Direction::compass()).unwrap();
    let (w, h, off_x, off_y) = match *side {
      Direction::North | Direction::South => {
        let offset_range = (room_w - DOOR_WIDTH - WALL_THICKNESS) / 2.0;
        let offset: f32 = rng.gen_range(-offset_range, offset_range);
        (DOOR_WIDTH, WALL_THICKNESS, offset, 0.0)
      }
      _ => {
        let offset_range = (room_h - DOOR_WIDTH - WALL_THICKNESS) / 2.0;
        let offset: f32 = rng.gen_range(-offset_range, offset_range);
        (WALL_THICKNESS, DOOR_WIDTH, 0.0, offset)
      }
    };
    let sidetup = side.to_tup();
    let door = Rect {
      x: c_x + sidetup.0 * (room_w / 2.0) - w / 2.0 + off_x,
      y: c_y + sidetup.1 * (room_h / 2.0) - h / 2.0 + off_y,
      w,
      h,
    };
    Room::new(Point::new(c_x, c_y), room_w, room_h, door, *side)
  }

  /// Generates walls for the room, appropriately making a gap for the door
  fn gen_walls(center: Point, width: Meters, height: Meters, door: Rect, door_side: Direction)
               -> Vec<Wall> {
    Direction::compass().iter().flat_map(|d| {
      let has_door = door_side == *d;
      let full_w = width + WALL_THICKNESS;
      let full_h = height + WALL_THICKNESS;
      match *d {
        Direction::North | Direction::South => {
          let yoffset = center.y + height / 2.0 * d.to_tup().1;
          let wall_c = Point::new(center.x, yoffset);
          if has_door {
            // Since door is ggez rect, x is left edge.
            let s1_rt_edge = door.x;
            let s1_lf_edge = center.x - width / 2.0;
            let s1c = Point::new(s1_lf_edge + (s1_rt_edge - s1_lf_edge) / 2.0, yoffset);
            let s2_rt_edge = center.x + width / 2.0;
            let s2_lf_edge = door.x + door.w;
            let s2c = Point::new(s2_lf_edge + (s2_rt_edge - s2_lf_edge) / 2.0, yoffset);
            let side1 = Wall::new(s1c, s1_rt_edge - s1_lf_edge, WALL_THICKNESS);
            let side2 = Wall::new(s2c, s2_rt_edge - s2_lf_edge, WALL_THICKNESS);
            vec![side1, side2]
          } else {
            vec![Wall::new(wall_c, full_w, WALL_THICKNESS)]
          }
        }
        _ => {
          let xoffset = center.x + width / 2.0 * d.to_tup().0;
          let wall_c = Point::new(xoffset, center.y);
          if has_door {
            // Since door is ggez rect, y is top edge.
            let s1_tp_edge = center.y - height / 2.0;
            let s1_bt_edge = door.y;
            let s1c = Point::new(xoffset, s1_tp_edge + (s1_bt_edge - s1_tp_edge) / 2.0);
            let s2_tp_edge = door.y + door.h;
            let s2_bt_edge = center.y + height / 2.0;
            let s2c = Point::new(xoffset, s2_tp_edge + (s2_bt_edge - s2_tp_edge) / 2.0);
            let side1 = Wall::new(s1c, WALL_THICKNESS, s1_bt_edge - s1_tp_edge);
            let side2 = Wall::new(s2c, WALL_THICKNESS, s2_bt_edge - s2_tp_edge);
            vec![side1, side2]
          } else {
            vec![Wall::new(wall_c, WALL_THICKNESS, full_h)]
          }
        }
      }
    }).collect()
  }

  /// Tests intersection with another room. Returns true if they intersect.
  pub fn intersects(&self, other: &Room) -> bool {
    let r1: Rect = self.into();
    let r2: Rect = other.into();
    !(r1.left() > r2.right() || r1.right() < r2.left() || r1.top() > r2.bottom()
      || r1.bottom() < r2.top())
  }

  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    for wall in &self.walls {
      let r: Rect = wall.into();
      rectangle(ctx, DrawMode::Fill, r)?;
    }
    set_color(ctx, Color::new(0.8, 0.8, 0.8, 1.0))?;
    rectangle(ctx, DrawMode::Fill, self.door)
  }
}

impl CenterOriginRect for Room {
  fn center(&self) -> Point { self.center }
  fn width(&self) -> f32 { self.width }
  fn height(&self) -> f32 { self.height }
}


impl<'a> From<&'a Room> for Rect {
  fn from(r: &Room) -> Rect {
    Rect {
      // GGEZ docs says x/y are center, they're actually top-left origin
      x: r.center().x - r.width() / 2.0,
      y: r.center().y - r.height() / 2.0,
      w: r.width(),
      h: r.height(),
    }
  }
}

impl<'a> From<&'a Wall> for Rect {
  fn from(r: &Wall) -> Rect {
    Rect {
      // GGEZ docs says x/y are center, they're actually top-left origin
      x: r.center().x - r.width() / 2.0,
      y: r.center().y - r.height() / 2.0,
      w: r.width(),
      h: r.height(),
    }
  }
}

impl<'a> Into<CollisionRect> for &'a Room {
  // Must be done as into b/c of generics
  fn into(self) -> CollisionRect {
    CollisionRect::new(Vector2::new(self.width / 2.0, self.height / 2.0))
  }
}

#[derive(new, Debug)]
struct Wall {
  center: Point,
  width: Meters,
  height: Meters,
}

impl CenterOriginRect for Wall {
  fn center(&self) -> Point { self.center }
  fn width(&self) -> f32 { self.width }
  fn height(&self) -> f32 { self.height }
}
