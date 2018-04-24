use collision::{Collidable, CollisionRect, Shape2D};
use ggez::{Context, GameResult};
use ggez::graphics::{Color, DrawMode, Rect, rectangle, set_color};
use na;
use na::{Isometry2, Vector2};
use nc;
use nc::ncollide_pipeline::world::CollisionGroups;
use nc::shape::{Compound2, ShapeHandle2};
use rand::{Rng, thread_rng};
use rand::distributions::{IndependentSample, Normal};
use super::direction::Direction;
use util::{Meters, Point};

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
  // TODO: Should probably just be a wall
  door: Rect,
  door_side: Direction,
  /// Tuple of wall, and side of the room that wall belongs to
  walls: Vec<(Wall, Direction)>,
}

impl Room {
  pub fn new(center: Point, width: Meters, height: Meters, door: Rect, door_side: Direction)
             -> Room {
    let walls = Room::gen_walls(center, width, height, door, door_side);
    Room { center, width, height, door, door_side, walls }
  }

  /// Creates a new `Room` randomly placed somewhere in the provided range
  pub fn new_rand((x_min, x_max): (Meters, Meters), (y_min, y_max): (Meters, Meters)) -> Room {
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    let (room_w, room_h) = Room::gen_room_box();
    let mut rng = thread_rng();
    // Add a door somewhere along the room edge
    let side = rng.choose(Direction::compass()).unwrap();
    let door = Room::gen_rand_door(c_x, c_y, room_w, room_h, side);
    Room::new(Point::new(c_x, c_y), room_w, room_h, door, *side)
  }

  /// Creates a new group of `Room`s that all touch each-other
  pub fn new_compound_room((x_min, x_max): (Meters, Meters), (y_min, y_max): (Meters, Meters))
                           -> Vec<Room> {
    let mut rng = thread_rng();
    // Start with an "anchor" room
    let anchor = Room::new_rand((x_min, x_max), (y_min, y_max));
    // Find the side the door is on, and tack on another room box there. We'll delete one wall
    // such that the two rooms now share a wall
    let prev_door = anchor.door;
    let prev_door_dir = anchor.door_side;
    let prev_door_c = Point::new(anchor.door.x + anchor.door.w / 2.0,
                                 anchor.door.y - anchor.door.h / 2.0);
    let (ext_w, ext_h) = Room::gen_room_box();
    let (ext_x, ext_y) = match prev_door_dir {
      Direction::North => (prev_door_c.x, anchor.center().y - anchor.height / 2.0 - ext_h / 2.0),
      Direction::South => (prev_door_c.x, anchor.center().y + anchor.height / 2.0 + ext_h / 2.0),
      Direction::East => (anchor.center().x + anchor.width / 2.0 + ext_w / 2.0, prev_door_c.y),
      Direction::West => (anchor.center().x - anchor.width / 2.0 - ext_w / 2.0, prev_door_c.y),
      _ => panic!("Impossible door side chosen during room generation"),
    };
    let dirs_no_same_side: Vec<&Direction> = Direction::compass().iter()
      .filter(|x| **x != prev_door_dir.opposite()).collect();
    let side = rng.choose(&dirs_no_same_side).unwrap();
    let door = Room::gen_rand_door(ext_x, ext_y, ext_w, ext_h, side);
    let mut extension = Room::new(Point::new(ext_x, ext_y), ext_w, ext_h, door, **side);
    // Generate walls as if we were using the door from the previous room, then use the walls
    // from that side in place of new room's "real" walls, so that we punch a hole where the door is
    let mut fixed_walls = Room::gen_walls(extension.center, ext_w, ext_h, prev_door,
                                          prev_door_dir.opposite());
    fixed_walls.retain(|&(_, d)| d == prev_door_dir.opposite());
    extension.walls.retain(|&(_, d)| d != prev_door_dir.opposite());
    extension.walls.append(&mut fixed_walls);

    vec![anchor, extension]
  }

  fn gen_room_box() -> (Meters, Meters) {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let (room_w, room_h) = {
      let sizer = Normal::new(5.0, 3.0);
      let mut get_siz = || {
        sizer
          .ind_sample(&mut rng)
          .abs()
          // Rooms must be at least 1m sq so a door can fit, plus a bit extra for wiggle room
          .max((DOOR_WIDTH + 0.2).into())
          .min(30.0) as Meters
      };
      (get_siz(), get_siz())
    };
    (room_w, room_h)
  }

  fn gen_rand_door(c_x: f32, c_y: f32, room_w: f32, room_h: f32, side: &Direction) -> Rect {
    let mut rng = thread_rng();
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
    door
  }

  /// Generates walls for the room, appropriately making a gap for the door
  fn gen_walls(center: Point, width: Meters, height: Meters, door: Rect, door_side: Direction)
               -> Vec<(Wall, Direction)> {
    Direction::compass().iter().flat_map(|d| {
      let d = *d; // This is a bit uggo
      let has_door = door_side == d;
      let full_w = width + WALL_THICKNESS;
      let full_h = height + WALL_THICKNESS;
      match d {
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
            vec![(side1, d), (side2, d)]
          } else {
            vec![(Wall::new(wall_c, full_w, WALL_THICKNESS), d)]
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
            vec![(side1, d), (side2, d)]
          } else {
            vec![(Wall::new(wall_c, WALL_THICKNESS, full_h), d)]
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
    for &(wall, _) in &self.walls {
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

impl From<Wall> for Rect {
  fn from(r: Wall) -> Rect {
    Rect {
      // GGEZ docs says x/y are center, they're actually top-left origin
      x: r.center().x - r.width() / 2.0,
      y: r.center().y - r.height() / 2.0,
      w: r.width(),
      h: r.height(),
    }
  }
}

impl Into<CollisionRect> for Wall {
  // Must be done as into b/c of generics
  fn into(self) -> CollisionRect {
    assert!(self.width > 0.0 && self.height > 0.0, "Wall has negative w or h! {:?}", self);
    CollisionRect::new(Vector2::new(self.width / 2.0, self.height / 2.0))
  }
}

// TODO: Doors, and how do we make them open/closed
impl Collidable for Room {
  fn location(&self) -> Point { self.center }
  fn shape(&self) -> Shape2D {
    let shapes = self.walls.iter().map(|&(w, _)| {
      let cr: CollisionRect = w.into();
      // Wall locations need to be represented relative to the center of the room
      let loc = Isometry2::new(w.center.coords - self.center.coords, na::zero());
      (loc, ShapeHandle2::new(cr))
    });
    let whole_shape = Compound2::new(shapes.collect());
    ShapeHandle2::new(whole_shape)
  }
  fn collision_group(&self) -> CollisionGroups {
    let mut cg = nc::world::CollisionGroups::new();
    cg.set_membership(&[1]);
    return cg;
  }
}

#[derive(new, Debug, PartialEq, Copy, Clone)]
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

// TESTS ================================================================================

#[cfg(test)]
mod test {
  #[test]
  fn test_wall_gen() {
    let c_x = 10.0;
    let c_y = 10.0;
    let w = 5.0;
    let h = 10.0;
    let side = Direction::North;
    let door = Room::gen_rand_door(c_x, c_y, w, h, &side);
    let walls = Room::gen_walls(Point::new(c_x, c_y), w, h, door, side);
    println!("{:?}", walls);
    // South wall (recall south is +y)
    assert_eq!(walls.len(), 5);
    let sw = Wall::new(Point::new(10.0, 15.0),
                       w + WALL_THICKNESS, WALL_THICKNESS);
    assert!(walls.contains(&sw));
    // West wall
    let ww = Wall::new(Point::new(7.5, 10.0),
                       WALL_THICKNESS, WALL_THICKNESS + h);
    assert!(walls.contains(&ww));
    // East wall
    let ew = Wall::new(Point::new(12.5, 10.0),
                       WALL_THICKNESS, WALL_THICKNESS + h);
    assert!(walls.contains(&ew));
    // I've stopped here because testing the north walls would involve duplicating a lot of
    // the existing logic. Revisit if refactored.
  }
}