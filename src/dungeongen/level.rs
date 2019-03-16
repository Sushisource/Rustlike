use super::blobstacle::Blobstacle;
use super::ca_simulator::CASim;
use super::direction::Direction;
use super::rooms::Room;
use crate::collision::{
  new_collw, CollW, Collidable, CollidableDat, CollidableType, GameObjRegistrar,
};
use crate::dungeongen::compound_room::CompoundRoomMaker;
use crate::nc::bounding_volume::AABB;
use crate::nc::shape::Polyline;
use crate::nc::world::CollisionObjectHandle;
use crate::util::context_help::ContextHelp;
use crate::util::geom::CenterOriginRect;
use crate::util::geom::CenteredRect;
use crate::util::{Meters, Point};
use ggez::graphics;
use ggez::graphics::{Color, DrawParam};
use ggez::{Context, GameResult};
use num::{FromPrimitive, ToPrimitive};
use rand::{thread_rng, Rng};

pub type Wall = CenteredRect;

pub static WALL_THICKNESS: Meters = 0.2;

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave_sim: CASim,
  pub level_gen_finished: bool,
  pub rooms: Vec<Room>,
  pub obstacles: Vec<Blobstacle>,
  gen_stage: LevelGenStage,
  width: Meters,
  height: Meters,
  /// This collision world can be used during different stages of level generation to
  /// make sure the stuff being generated isn't colliding with other stuff.
  tmp_collw: CollW,
  tmp_ent_ct: usize,
}

#[derive(PartialEq, Ord, PartialOrd, Eq, FromPrimitive, ToPrimitive)]
enum LevelGenStage {
  CaveSim,
  RoomSim,
  PlaceObstacles,
  Done,
}

impl Level {
  pub fn new() -> Level {
    Level {
      // TODO: Right now the dimensions of this sim need to have the same ratio
      // as the screen or it gets squished. It's also bad at taking up most of the available screen
      // space.
      cave_sim: CASim::new(266, 150, 1.0),
      level_gen_finished: false,
      rooms: Vec::new(),
      obstacles: Vec::new(),
      gen_stage: LevelGenStage::CaveSim,
      width: 80.0,
      height: 45.0,
      tmp_collw: new_collw(),
      tmp_ent_ct: 0,
    }
  }

  pub fn tick_level_gen(&mut self) -> () {
    let stage_complete = match self.gen_stage {
      LevelGenStage::CaveSim => self.tick_cavesim(),
      LevelGenStage::RoomSim => self.tick_roomsim(),
      LevelGenStage::PlaceObstacles => self.place_obstacles(),
      _ => false,
    };
    if stage_complete {
      self.gen_stage = ToPrimitive::to_u8(&self.gen_stage)
        .and_then(|v| FromPrimitive::from_u8(v + 1))
        .unwrap_or(LevelGenStage::Done);
    }
    if self.gen_stage == LevelGenStage::Done {
      self.level_gen_finished = true;
    }
  }

  fn tick_cavesim(&mut self) -> bool {
    self.cave_sim.tick()
  }

  fn tick_roomsim(&mut self) -> bool {
    let mut rng = thread_rng();
    // Room centers should be within the bounding box of the cave
    let cave_bb = self.cave_bound_box();
    let xrange = (cave_bb.mins().x, cave_bb.maxs().x);
    let yrange = (cave_bb.mins().y, cave_bb.maxs().y);
    if self.rooms.is_empty() {
      // First run through add the cave BB to the collision world so we don't get rooms too far
      // outside of the cave. To get the four walls, it's easy to convert the BB into a "room".
      let cave_bb_room = Room::new_with_centered_door(
        cave_bb.center(),
        cave_bb.half_extents().x * 2.0,
        cave_bb.half_extents().y * 2.0,
        Direction::North,
      )
      .unwrap();
      let nxt_id = self.get_and_inc_eid();
      self.tmp_collw.register(&cave_bb_room, CollidableDat::new(cave_bb.coltype(), nxt_id));
      self.tmp_collw.update();
    }
    // TODO: Change back to more rooms / not 100% compound rooms
    if self.rooms.len() < 1 {
      loop {
        let is_compound = rng.gen_bool(5.0 / 5.0);
        let mut nu_rooms = Vec::new();
        if is_compound {
          if let Ok(mut room) = CompoundRoomMaker::rand_compound_room(xrange, yrange) {
            nu_rooms.append(&mut room);
          } else {
            // If we failed to generate a compound room, restart and generate a new room
            continue;
          }
        } else {
          nu_rooms.push(Room::new_rand(xrange, yrange));
        }
        let cw_typ =
          if is_compound { CollidableType::CompoundRoomWall } else { CollidableType::RoomWall };
        let cw_dat = CollidableDat::new(cw_typ, self.get_and_inc_eid());
        let (coll_handles, no_collisions) =
          Level::check_room_collisions(&mut self.tmp_collw, &nu_rooms, cw_dat);
        if no_collisions {
          self.rooms.append(&mut nu_rooms);
          break;
        } else {
          self.tmp_collw.remove(coll_handles.as_slice());
        }
      }
      false
    } else {
      info!("Done placing rooms");
      true
    }
  }

  /// Returns a tuple of (collision handles, were any collisions)
  fn check_room_collisions(
    collw: &mut CollW,
    nu_rooms: &[Room],
    cw_dat: CollidableDat,
  ) -> (Vec<CollisionObjectHandle>, bool) {
    let coll_handles: Vec<CollisionObjectHandle> = nu_rooms
      .iter()
      .flat_map(|nr| {
        let mut handles = vec![collw.register(nr, cw_dat)];
        let floormats = nr.floormat();
        handles.extend(floormats.iter().map(|f| collw.register(&(f as &CenterOriginRect), cw_dat)));
        handles
      })
      .collect();
    collw.update();
    (coll_handles, has_no_collisions(collw))
  }

  pub fn cave_bound_box(&self) -> AABB<Meters> {
    let cavebf: Vec<Point> = self.cave_bounds();
    let cave_polyline = Polyline::new(cavebf, None);
    cave_polyline.aabb().clone()
  }

  fn cave_bounds(&self) -> Vec<Point> {
    self
      .cave_sim
      .uspace_boundary(Point::new(0.0, 0.0))
      .iter()
      .map(|&p| self.uspace_to_lspace(p))
      .collect()
  }

  fn place_obstacles(&mut self) -> bool {
    // Grow some ponds using our CA generation method
    // TODO: Re-enable / do something actually useful when I get to this point
    //    let test_pond = Blobstacle::new(Point::new(30.0, 30.0));
    //    let test_pond2 = Blobstacle::new(Point::new(5.5, 5.1));
    //    let test_pond3 = Blobstacle::new(Point::new(20.8, 20.8));
    //    self.obstacles.push(test_pond);
    //    self.obstacles.push(test_pond2);
    //    self.obstacles.push(test_pond3);
    true
  }

  /// Converts level space to unit space
  pub fn lspace_to_uspace(&self, p: Point) -> Point {
    Point::new(p.x / self.width, p.y / self.height)
  }

  /// Converts unit space to level space
  pub fn uspace_to_lspace(&self, p: Point) -> Point {
    Point::new(p.x * self.width, p.y * self.height)
  }

  pub fn middle(&self) -> Point {
    Point::new(self.width / 2.0, self.height / 2.0)
  }

  pub fn produce_collidables(&self) -> Vec<&Collidable> {
    self.rooms.iter().map(|r| r as &Collidable).collect()
  }

  fn get_and_inc_eid(&mut self) -> usize {
    let c_id = self.tmp_ent_ct;
    self.tmp_ent_ct += 1;
    c_id
  }

  // Rendering code below =============================================================
  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    graphics::set_transform(ctx, DrawParam::default().to_matrix());
    graphics::apply_transformations(ctx)?;
    let sscale = ctx.sscale();
    let center_scale = self.lscale(ctx);

    if self.gen_stage == LevelGenStage::CaveSim {
      self.cave_sim.draw_evolution(ctx, sscale)?;
    } else {
      graphics::set_transform(ctx, center_scale.to_matrix());
      graphics::apply_transformations(ctx)?;
      // Next stage, we render the cave as a polygon and place rooms
      let color = Color::new(0.5, 0.5, 0.5, 1.0);
      // TODO: We also do this u->l conversion in the generator. Combine
      // somehow?
      self.cave_sim.draw(ctx, self.u_to_l_scale().color(color))?;

      if !self.rooms.is_empty() {
        for room in &self.rooms {
          let grayval = 0.3;
          room.draw(ctx, &DrawParam::new().color(Color::new(grayval, grayval, grayval, 1.0)))?;
        }
      }

      if !self.rooms.is_empty() {
        for obstacle in &self.obstacles {
          obstacle.draw(ctx)?;
        }
      }
      //       Test center room of one sq unit
      //      graphics::set_color(ctx, Color::new(0.0, 0.5, 0.0, 1.0))?;
      //      ctx.center_rect(self.middle(), 1.0, 1.0)?;
    }
    Ok(())
  }

  fn lspace_to_sspace(&self, ctx: &Context, p: Point) -> Point {
    let p = self.lspace_to_uspace(p);
    ctx.uspace_to_sspace(p)
  }

  pub fn sspace_to_lspace(&self, ctx: &Context, p: Point) -> Point {
    let p = ctx.sspace_to_uspace(p);
    self.uspace_to_lspace(p)
  }

  fn u_to_l_scale(&self) -> DrawParam {
    let as_vec = self.uspace_to_lspace(Point::new(1.0, 1.0)).coords;
    DrawParam { scale: as_vec.into(), ..Default::default() }
  }

  pub fn lscale(&self, ctx: &Context) -> DrawParam {
    let as_vec = self.lspace_to_sspace(ctx, Point::new(1.0, 1.0)).coords;
    DrawParam { scale: as_vec.into(), ..Default::default() }
  }
}

fn has_no_collisions(collw: &CollW) -> bool {
  collw.contact_pairs(true).peekable().peek().is_none()
  //.any(|p| p.2.num_contacts() > 0)
}

#[cfg(test)]
mod test {
  extern crate timebomb;

  use self::timebomb::timeout_ms;
  use super::*;

  #[test]
  fn test_no_room_collisions() {
    timeout_ms(
      || {
        let mut l = Level::new();
        while l.gen_stage < LevelGenStage::PlaceObstacles {
          l.tick_level_gen();
        }
        l.tmp_collw.update();
        assert!(has_no_collisions(&l.tmp_collw))
      },
      10000,
    )
  }

  #[test]
  fn test_rooms_can_nest() {
    let mut collw = new_collw();
    let room1 =
      Room::new_with_centered_door(Point::new(0.0, 0.0), 10.0, 10.0, Direction::South).unwrap();
    let dat1 = CollidableDat::new(CollidableType::RoomWall, 1);
    let room2 =
      Room::new_with_centered_door(Point::new(0.0, 0.0), 3.0, 3.0, Direction::North).unwrap();
    let dat2 = CollidableDat::new(CollidableType::RoomWall, 2);
    Level::check_room_collisions(&mut collw, &vec![room1], dat1);
    let (_, no_collisions) = Level::check_room_collisions(&mut collw, &vec![room2], dat2);
    assert!(no_collisions);
  }

  #[test]
  fn test_room_floormats_work() {
    let mut collw = new_collw();
    // The bottom wall of this room is @ 2.0
    let room1 =
      Room::new_with_centered_door(Point::new(0.0, 0.0), 4.0, 4.0, Direction::South).unwrap();
    let dat1 = CollidableDat::new(CollidableType::RoomWall, 1);
    // The top wall of this room is @ 2.25 (too close)
    let room2 =
      Room::new_with_centered_door(Point::new(0.0, 4.0), 4.0, 3.5, Direction::East).unwrap();
    let dat2 = CollidableDat::new(CollidableType::RoomWall, 2);
    let (_, no_collisions) = Level::check_room_collisions(&mut collw, &vec![room1], dat1);
    assert!(no_collisions);
    let (_, no_collisions) = Level::check_room_collisions(&mut collw, &vec![room2], dat2);
    // There should be collisions!
    assert!(!no_collisions);
  }
}
