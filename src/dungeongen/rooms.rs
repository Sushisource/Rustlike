use super::direction::Direction;
use collision::{CollGroups, Collidable, CollidableType, CollisionRect, Shape2D};
use ggez::graphics::{rectangle, set_color, Color, DrawMode, Rect};
use ggez::{Context, GameResult};
use na;
use na::{Isometry2, Vector2};
use nc::shape::{Compound, ShapeHandle};
use nc::world::CollisionGroups;
use rand::distributions::{Distribution, Normal};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;
use util::geom::{snap_to_existing_rooms, CenterOriginRect, CenteredRect, GridRect, IntPoint};
use util::{Meters, Point};

static WALL_THICKNESS: Meters = 0.2;
static DOOR_WIDTH: Meters = 1.1;

#[derive(Debug, CenterOriginRect)]
pub struct Room {
  cr: CenteredRect,
  door: Door,
  /// Tuple of wall, and side of the room that wall belongs to
  walls: Vec<(Wall, Direction)>,
  is_compound: bool,
}

impl<'a> Into<Room> for &'a GridRect {
  fn into(self) -> Room {
    let nc: Point = na::convert(self.center());
    // TODO: Door stuff.
    Room::new(
      nc,
      self.width as f32,
      self.height as f32,
      Door::new(CenteredRect::new(nc, 0.1, 0.1), Direction::North),
      true,
    )
  }
}

pub type CompoundRoom = Vec<Room>;

impl Room {
  pub fn new(center: Point, width: Meters, height: Meters, door: Door, is_compound: bool) -> Room {
    let walls = Room::gen_walls(center, width, height, door, door.facing);
    Room { cr: CenteredRect::new(center, width, height), door, walls, is_compound }
  }

  /// Creates a new `Room` randomly placed somewhere in the provided range
  pub fn new_rand((x_min, x_max): (Meters, Meters), (y_min, y_max): (Meters, Meters)) -> Room {
    let mut rng = thread_rng();
    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    let (room_w, room_h) = Room::rand_room_box();
    let mut rng = thread_rng();
    // Add a door somewhere along the room edge
    let side = rng.choose(Direction::compass()).unwrap();
    let door = Room::gen_rand_door(c_x, c_y, room_w, room_h, *side);
    Room::new(Point::new(c_x, c_y), room_w, room_h, door, false)
  }

  /// Creates a new group of `Room`s that all touch each-other. This is done in a gridded space
  /// to allow snapping rooms together precisely. Parameters are max/min sizes for an individual
  /// room within the compound room.
  pub fn rand_compound_room(
    (x_min, x_max): (Meters, Meters),
    (y_min, y_max): (Meters, Meters),
  ) -> CompoundRoom {
    let mut rng = thread_rng();
    // The initial room
    let starter = Room::rand_grid_room();

    let mut rooms = vec![starter];
    let num_extensions = rng.gen_range(1, 5);

    for _ in 0..num_extensions {
      let exit_angle = rng.gen_range(0.0, PI * 2.0);
      let new = Room::rand_grid_room();
      let moved = snap_to_existing_rooms(&rooms, &new, exit_angle);
      rooms.push(moved);
    }

    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    for r in rooms.iter_mut() {
      r.top_left.x += c_x as i32;
      r.top_left.y += c_y as i32;
    }

    rooms.iter().map(|r| r.into()).collect()
  }

  /// Creates a new `Room` with a door centered along the wall of the provided direction
  pub fn new_with_centered_door(
    center: Point,
    width: Meters,
    height: Meters,
    door_side: Direction,
  ) -> Room {
    let door = Room::gen_door(center.x, center.y, width, height, door_side, 0.0);
    Room::new(center, width, height, door, false)
  }

  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    for &(wall, _) in &self.walls {
      let r: Rect = (&wall as &CenterOriginRect).into();
      rectangle(ctx, DrawMode::Fill, r)?;
    }
    set_color(ctx, Color::new(0.8, 0.8, 0.8, 1.0))?;
    rectangle(ctx, DrawMode::Fill, (&self.door as &CenterOriginRect).into())
  }

  /// Returns a collidable that can be used during room placement to ensure there is enough space
  /// on either side of the Room's door to accommodate the player.
  pub fn floormat(&self) -> CenteredRect {
    let wider = self.door.width() > self.door.height();
    let expander = if wider { (0.0, DOOR_WIDTH * 1.5) } else { (DOOR_WIDTH * 1.5, 0.0) };
    CenteredRect::new(
      self.door.center(),
      self.door.width() + expander.0,
      self.door.height() + expander.1,
    )
  }

  /// Creates a randomly sized grid room with top-left corner at origin
  fn rand_grid_room() -> GridRect {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let (room_w, room_h) = {
      let sizer = Normal::new(5.0, 3.0);
      let mut get_siz = || {
        sizer
          .sample(&mut rng)
          .abs()
          // Rooms need to be big enough to fit a door, and a little wiggle room
          .max((DOOR_WIDTH * 2.0).into())
          .min(30.0) as u32
      };
      (get_siz(), get_siz())
    };
    GridRect::new(room_w, room_h, IntPoint::new(0, 0))
  }

  fn rand_room_box() -> (Meters, Meters) {
    // TODO: Configurable sizing parameters
    let mut rng = thread_rng();
    let (room_w, room_h) = {
      let sizer = Normal::new(5.0, 3.0);
      let mut get_siz = || {
        sizer
          .sample(&mut rng)
          .abs()
          // Rooms need to be big enough to fit a door, and a little wiggle room
          .max((DOOR_WIDTH * 2.0).into())
          .min(30.0) as Meters
      };
      (get_siz(), get_siz())
    };
    (room_w, room_h)
  }

  fn gen_rand_door(c_x: f32, c_y: f32, room_w: f32, room_h: f32, side: Direction) -> Door {
    let mut rng = thread_rng();
    let offset_mul: f32 = rng.gen_range(-1.0, 1.0);
    Room::gen_door(c_x, c_y, room_w, room_h, side, offset_mul)
  }

  /// Non-random door generation. `offset_multiplier` here is a value between -1.0 and 1.0
  fn gen_door(
    c_x: f32,
    c_y: f32,
    room_w: f32,
    room_h: f32,
    side: Direction,
    offset_multiplier: f32,
  ) -> Door {
    let (w, h, off_x, off_y) = match side {
      Direction::North | Direction::South => {
        let offset = ((room_w - DOOR_WIDTH - WALL_THICKNESS) / 2.0) * offset_multiplier;
        (DOOR_WIDTH, WALL_THICKNESS, offset, 0.0)
      }
      _ => {
        let offset = ((room_h - DOOR_WIDTH - WALL_THICKNESS) / 2.0) * offset_multiplier;
        (WALL_THICKNESS, DOOR_WIDTH, 0.0, offset)
      }
    };
    let sidetup = side.to_tup();
    Door::new(
      CenteredRect::new(
        Point::new(
          c_x + sidetup.0 * (room_w / 2.0) + off_x,
          c_y + sidetup.1 * (room_h / 2.0) + off_y,
        ),
        w,
        h,
      ),
      side,
    )
  }

  /// Generates walls for the room, appropriately making a gap for the door. `door_side` must
  /// be passed even though `door` has a `facing` property, because you may want to punch a hole
  /// in some walls using another room's door.
  fn gen_walls(
    center: Point,
    width: Meters,
    height: Meters,
    door: Door,
    door_side: Direction,
  ) -> Vec<(Wall, Direction)> {
    Direction::compass()
      .iter()
      .flat_map(|d| {
        let d = *d;
        let has_door = door_side == d;
        let full_w = width + WALL_THICKNESS;
        let full_h = height + WALL_THICKNESS;
        match d {
          Direction::North | Direction::South => {
            let yoffset = center.y + height / 2.0 * d.to_tup().1;
            let wall_c = Point::new(center.x, yoffset);
            if has_door {
              let s1_rt_edge = door.left_edge();
              let s1_lf_edge = center.x - width / 2.0 - WALL_THICKNESS / 2.0;
              let s1c = Point::new(s1_lf_edge + (s1_rt_edge - s1_lf_edge) / 2.0, yoffset);
              let s2_rt_edge = center.x + width / 2.0 + WALL_THICKNESS / 2.0;
              let s2_lf_edge = door.right_edge();
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
              let s1_tp_edge = center.y - height / 2.0 - WALL_THICKNESS / 2.0;
              let s1_bt_edge = door.top_edge();
              let s1c = Point::new(xoffset, s1_tp_edge + (s1_bt_edge - s1_tp_edge) / 2.0);
              let s2_tp_edge = door.bottom_edge();
              let s2_bt_edge = center.y + height / 2.0 + WALL_THICKNESS / 2.0;
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
}

type Wall = CenteredRect;

impl Into<CollisionRect> for Wall {
  // Must be done as into b/c of generics
  fn into(self) -> CollisionRect {
    CollisionRect::new(Vector2::new(self.width() / 2.0, self.height() / 2.0))
  }
}

impl Collidable for Room {
  fn location(&self) -> Isometry2<Meters> {
    Isometry2::new(self.center().coords, na::zero())
  }
  fn shape(&self) -> Shape2D {
    let shapes = self.walls.iter().map(|&(w, _)| {
      let cr: CollisionRect = w.into();
      // Wall locations need to be represented relative to the center of the room
      let loc = Isometry2::new(w.center().coords - self.center().coords, na::zero());
      (loc, ShapeHandle::new(cr))
    });
    let whole_shape = Compound::new(shapes.collect());
    ShapeHandle::new(whole_shape)
  }
  fn collision_group(&self) -> CollisionGroups {
    CollGroups::wall_cg()
  }
  fn coltype(&self) -> CollidableType {
    if self.is_compound {
      CollidableType::CompoundRoomWall
    } else {
      CollidableType::RoomWall
    }
  }
}

#[derive(new, Debug, PartialEq, Copy, Clone, CenterOriginRect)]
pub struct Door {
  cr: CenteredRect,
  facing: Direction,
}

// TESTS ================================================================================

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_wall_gen() {
    let c_x = 10.0;
    let c_y = 10.0;
    let w = 5.0;
    let h = 10.0;
    let side = Direction::North;
    let door = Room::gen_door(c_x, c_y, w, h, side, 0.0);
    let walls = Room::gen_walls(Point::new(c_x, c_y), w, h, door, side);
    println!("{:?}", walls);
    // South wall (recall south is +y)
    assert_eq!(walls.len(), 5);
    let sw = Wall::new(Point::new(10.0, 15.0), w + WALL_THICKNESS, WALL_THICKNESS);
    assert!(walls.contains(&(sw, Direction::South)));
    // West wall
    let ww = Wall::new(Point::new(7.5, 10.0), WALL_THICKNESS, WALL_THICKNESS + h);
    assert!(walls.contains(&(ww, Direction::West)));
    // East wall
    let ew = Wall::new(Point::new(12.5, 10.0), WALL_THICKNESS, WALL_THICKNESS + h);
    assert!(walls.contains(&(ew, Direction::East)));
    // North walls
    let west_edge = c_x - w / 2.0 - WALL_THICKNESS / 2.0;
    let nw1 = Wall::new(
      Point::new(west_edge + (door.left_edge() - west_edge) / 2.0, 5.0),
      door.left_edge() - west_edge,
      WALL_THICKNESS,
    );
    assert!(walls.contains(&(nw1, Direction::North)));
    let east_edge = c_x + w / 2.0 + WALL_THICKNESS / 2.0;
    let door_rt = door.right_edge();
    let nw2 = Wall::new(
      Point::new(door_rt + (east_edge - door_rt) / 2.0, 5.0),
      east_edge - door_rt,
      WALL_THICKNESS,
    );
    assert!(walls.contains(&(nw2, Direction::North)));
  }
}
