use collision::{Collidable, CollisionRect};
use dungeongen::level::WALL_THICKNESS;
use dungeongen::rooms::DOOR_WIDTH;
use dungeongen::{direction::Direction, rooms::Door, rooms::Room};
use na;
use na::{Isometry2, Vector2};
use nc::{query, query::Contact, shape::Compound};
use num::abs;
use rand::distributions::{Distribution, Normal};
use rand::thread_rng;
use rand::Rng;
use std::f32::consts::PI;
use util::Point;
use util::{
  geom::{origin, walk_grid, CenterOriginRect, CenteredRect, GridRect, IntPoint, PolarVec},
  Meters,
};

pub type CompoundRoom = Vec<Room>;

pub struct CompoundRoomMaker {
  rects: Vec<GridRect>,
  rooms: Vec<Room>,
}

impl CompoundRoomMaker {
  pub fn new(starter_rect: GridRect) -> CompoundRoomMaker {
    let starter_center: Point = na::convert(starter_rect.center());
    let rooms = vec![
      CompoundRoomMaker::grid_room_to_room(
        &starter_rect,
        Door::new(CenteredRect::new(starter_center, 1.0, 1.0), Direction::North),
      ).unwrap(),
    ];
    CompoundRoomMaker { rects: vec![starter_rect], rooms }
  }
  /// Creates a new group of `Room`s that all touch each-other. This is done in a gridded space
  /// to allow snapping rooms together precisely. Parameters are max/min sizes for an individual
  /// room within the compound room.
  pub fn rand_compound_room(
    (x_min, x_max): (Meters, Meters),
    (y_min, y_max): (Meters, Meters),
  ) -> Result<CompoundRoom, ()> {
    let mut rng = thread_rng();
    // The initial room
    let starter = CompoundRoomMaker::rand_grid_room();

    let mut maker = CompoundRoomMaker::new(starter);

    let num_extensions = rng.gen_range(1, 5);

    for _ in 0..num_extensions {
      let exit_angle = rng.gen_range(0.0, PI * 2.0);
      let new = CompoundRoomMaker::rand_grid_room();
      let contact = maker.snap_to_existing_rooms(&new, exit_angle);
      let moved_room = maker.rects.last().unwrap();
      debug!("ROOM: {:?}\nCONTACT: {:?}", moved_room, contact);
      let midpt = maker.find_wall_overlap_midpoint(&contact)?;
      debug!("MIDP: {:?}", midpt);
      // Punch a door between this new room and whatever room it is contacting
      let contact_dir = Direction::from_normal(contact.normal.as_slice());
      let door = Door::of_width(midpt, DOOR_WIDTH, contact_dir);
      maker.rooms.push(CompoundRoomMaker::grid_room_to_room(&moved_room, door)?);
    }

    // Punch doors to the outside where necessary to make all rooms accessible

    // Shift all the rooms into a randomly selected position
    let c_x: f32 = rng.gen_range(x_min, x_max);
    let c_y: f32 = rng.gen_range(y_min, y_max);
    for r in maker.rooms.iter_mut() {
      r.translate(c_x, c_y);
    }

    Ok(maker.rooms)
  }

  /// Given a point of contact, searches all existing rects for two walls which are axis-aligned
  /// and have the contact point in that line and within the bounds of the walls.
  ///
  /// Returns the midpoint of the overlap of the walls
  ///
  /// Can fail if overlap area isn't big enough, or contact point doesn't include two walls
  fn find_wall_overlap_midpoint(&self, contact: &Contact<Meters>) -> Result<Point, ()> {
    let all_walls: Vec<(CenteredRect, Direction)> = self
      .rects
      .iter()
      .flat_map(|r| {
        let cr = r as &CenterOriginRect;
        cr.gen_walls().into_iter()
      }).collect();
    debug!("All walls: {:?}", all_walls);
    let contact_dir = Direction::from_normal(contact.normal.as_slice());
    debug!("Contact dir: {:?}", contact_dir);
    let target_walls: Vec<_> = all_walls
      .into_iter()
      // Find walls parallel to the contact's normal
      .filter(|(w, d)| {
        (*d == contact_dir || d.opposite() == contact_dir) &&
          // And in the correct position
          if *d == Direction::East || *d == Direction::West {
            abs(w.center.x - contact.world1.x) < 0.0001 &&
              (w.bottom_edge() > contact.world1.y && contact.world1.y > w.top_edge())
          } else {
            abs(w.center.y - contact.world1.y) < 0.0001 &&
              (w.left_edge() < contact.world1.x && contact.world1.x < w.right_edge())
          }
      }).collect();
    debug!("target walls: {:?}", target_walls);
    if target_walls.len() != 2 {
      warn!("Couldn't find two walls that worked when punching door in compound room");
      return Err(());
    };
    // Find the overlap of the target walls
    let (w1, d) = target_walls[0];
    let (w2, _) = target_walls[1];
    // Sort all edges, and the #2 and #3 items are the endpoints of the overlapping segment
    let mut sorted_edges = if d == Direction::East || d == Direction::West {
      vec![w1.bottom_edge(), w1.top_edge(), w2.bottom_edge(), w2.top_edge()]
    } else {
      vec![w1.left_edge(), w1.right_edge(), w2.left_edge(), w2.right_edge()]
    };
    sorted_edges.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (low_end, hi_end) = if d == Direction::East || d == Direction::West {
      (sorted_edges[2], sorted_edges[1])
    } else {
      (sorted_edges[1], sorted_edges[2])
    };
    let size = abs(hi_end - low_end);
    if size < DOOR_WIDTH + WALL_THICKNESS * 2.0 {
      warn!("Wall overlap not big enough to punch door");
      return Err(());
    }
    let midpoint = (hi_end + low_end) / 2.0;
    debug!("lo {:?} hi {:?}", low_end, hi_end);
    if d == Direction::East || d == Direction::West {
      Ok(Point::new(w1.center.x, midpoint))
    } else {
      Ok(Point::new(midpoint, w1.center.y))
    }
  }

  /// Given some existing rooms (clustered around the origin) and a new room (at the origin),
  /// move the new room away from the origin at the exit angle until flush with the edge of one
  /// of the existing rooms. Rooms can't be bigger than 1000 meters in any direction.
  fn snap_to_existing_rooms(&mut self, new_room: &GridRect, exit_angle: f32) -> Contact<Meters> {
    let walk_vec: Vector2<Meters> = PolarVec::new(1000.0, exit_angle).into();
    let walk_to_pt = IntPoint::new(walk_vec.x as i32, walk_vec.y as i32);
    let walk_list = walk_grid(IntPoint::new(0, 0), walk_to_pt);
    let orig_w = new_room.width;
    let orig_h = new_room.height;
    let nr_coll = new_room as &CenterOriginRect;
    let nr_shape1 = nr_coll.shape();
    let nr_shape2: &CollisionRect = nr_shape1.as_shape().unwrap();
    let nr_shape_half_w = nr_shape2.half_extents().x;
    let nr_shape_half_h = nr_shape2.half_extents().y;
    let nr_shape: CollisionRect =
      CollisionRect::new(Vector2::new(nr_shape_half_w, nr_shape_half_h));
    let compound_shape = {
      let room_rects: Vec<&CenterOriginRect> =
        self.rects.iter().map(|r| r as &CenterOriginRect).collect();
      compoundify(&room_rects)
    };
    let mut last_pt = nr_coll.location();
    let mut last_contact = None;
    for walkpt in walk_list {
      let cur_pt = Isometry2::new(
        Vector2::new(walkpt.x as f32 + nr_shape_half_w, walkpt.y as f32 + nr_shape_half_h),
        0.0,
      );
      let contact = query::contact(&origin(), &compound_shape, &cur_pt, &nr_shape, 0.0);
      let is_contact = contact.is_some();

      if !is_contact {
        break;
      } else {
        last_contact = contact;
      }
      // Same as cur pt without the center shift
      last_pt = Isometry2::new(Vector2::new(walkpt.x as f32, walkpt.y as f32), 0.0);
    }
    let intified: Vector2<i32> =
      Vector2::new(last_pt.translation.vector.x as i32, last_pt.translation.vector.y as i32);
    self.rects.push(GridRect::new(orig_w, orig_h, IntPoint::from(intified)));
    return last_contact.unwrap();
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
          .max((DOOR_WIDTH * 2.0 + 0.2).into())
          .min(30.0) as u32
      };
      (get_siz(), get_siz())
    };
    GridRect::new(room_w, room_h, IntPoint::new(0, 0))
  }

  fn grid_room_to_room(gr: &GridRect, door: Door) -> Result<Room, ()> {
    let nc: Point = na::convert(gr.center());
    Room::new(nc, gr.width as f32, gr.height as f32, door, true)
  }
}

fn compoundify(shapes: &[impl Collidable]) -> Compound<f32> {
  Compound::new(shapes.iter().map(|r| (r.location(), r.shape())).collect())
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_simple_snap() {
    let mut maker = CompoundRoomMaker::new(GridRect::new(1, 1, IntPoint::new(0, 0)));
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, 0.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(1, 0)), *maker.rects.last().unwrap());
  }

  #[test]
  fn test_series_of_snaps() {
    let mut maker = CompoundRoomMaker::new(GridRect::new(1, 1, IntPoint::new(0, 0)));
    // Up
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, PI / 2.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(0, 1)), *maker.rects.last().unwrap());
    // Right
    let new = GridRect::new(4, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, 0.0);
    assert_eq!(GridRect::new(4, 1, IntPoint::new(1, 0)), *maker.rects.last().unwrap());
    // Up again, two more times
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, PI / 2.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(0, 2)), *maker.rects.last().unwrap());

    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, PI / 2.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(0, 3)), *maker.rects.last().unwrap());
    // Diagonally up and right
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, PI / 4.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(1, 1)), *maker.rects.last().unwrap());
    // Right again
    let new = GridRect::new(1, 1, IntPoint::new(0, 0));
    maker.snap_to_existing_rooms(&new, 0.0);
    assert_eq!(GridRect::new(1, 1, IntPoint::new(5, 0)), *maker.rects.last().unwrap());
  }
}
