use collision::{CollidableDat, CollidableType, CollW, GameObjRegistrar, new_collw, Collidable};
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::graphics::{Color, DrawParam};
use na;
use na::{Isometry2, Point2};
use nc::bounding_volume::AABB;
use nc::ncollide_pipeline::world::CollisionObjectHandle;
use nc::shape::{Polyline, Shape};
use num::{FromPrimitive, ToPrimitive};
use rand::{Rng, thread_rng};
use std::sync::Arc;
use super::blobstacle::Blobstacle;
use super::ca_simulator::CASim;
use super::rooms::Room;
use util::{Meters, Point};
use util::context_help::ContextHelp;

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
    if self.rooms.len() < 20 {
      loop {
        let is_compound = rng.gen_weighted_bool(5);
        let mut nu_rooms = Vec::new();
        if is_compound {
          nu_rooms.append(&mut Room::new_compound_room(xrange, yrange))
        } else {
          nu_rooms.push(Room::new_rand(xrange, yrange));
        }
        let cw_typ =
          if is_compound { CollidableType::CompoundRoomWall } else { CollidableType::RoomWall };
        let cw_dat = CollidableDat::new(cw_typ, self.get_and_inc_eid());
        let coll_handles: Vec<CollisionObjectHandle> =
          nu_rooms.iter().map(|nr| self.tmp_collw.register(nr, cw_dat)).collect();
        self.tmp_collw.update();
        let no_collisions = self.tmp_collw.contacts().next().is_none();
        if no_collisions {
          self.rooms.append(&mut nu_rooms);
          break;
        } else {
          self.tmp_collw.remove(coll_handles.as_slice())
        }
      }
      false
    } else {
      println!("Done placing rooms");
      true
    }
  }

  pub fn cave_bound_box(&self) -> AABB<Point> {
    let cavebf: Vec<Point> = self.cave_bounds();
    let cave_ixs: Vec<Point2<usize>> = (0..cavebf.len())
      .map(|i| {
        let to = if i + 1 == cavebf.len() { 0 } else { i + 1 };
        Point2::new(i, to)
      })
      .collect();
    let cave_polyline =
      Polyline::new(Arc::new(cavebf), Arc::new(cave_ixs), None, None);
    let cave_pos = na::one::<Isometry2<Meters>>();
    cave_polyline.aabb(&cave_pos)
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
    let test_pond = Blobstacle::new(Point::new(30.0, 30.0));
    let test_pond2 = Blobstacle::new(Point::new(5.5, 5.1));
    let test_pond3 = Blobstacle::new(Point::new(20.8, 20.8));
    self.obstacles.push(test_pond);
    self.obstacles.push(test_pond2);
    self.obstacles.push(test_pond3);
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
    graphics::set_transform(ctx, DrawParam::default().into_matrix());
    graphics::apply_transformations(ctx)?;
    let sscale = ctx.sscale();
    let center_scale = self.lscale(ctx);

    if self.gen_stage == LevelGenStage::CaveSim {
      &self.cave_sim.draw_evolution(ctx, sscale);
    } else {
      graphics::set_transform(ctx, center_scale.into_matrix());
      graphics::apply_transformations(ctx)?;
      // Next stage, we render the cave as a polygon and place rooms
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      // TODO: We also do this u->l conversion in the generator. Combine
      // somehow?
      self.cave_sim.draw(ctx, self.u_to_l_scale())?;

      if self.rooms.len() > 0 {
        for room in &self.rooms {
          let grayval = 0.2;
          graphics::set_color(ctx, Color::new(grayval, grayval, grayval, 1.0))?;
          room.draw(ctx)?;
        }
      }

      if self.obstacles.len() > 0 {
        for obstacle in &self.obstacles {
          graphics::set_color(ctx, (227, 77, 40).into())?;
          obstacle.draw(ctx)?;
        }
      }
      // Test center room of one sq unit
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
    DrawParam {
      scale: self.uspace_to_lspace(Point::new(1.0, 1.0)),
      ..Default::default()
    }
  }

  pub fn lscale(&self, ctx: &Context) -> DrawParam {
    return DrawParam {
      scale: self.lspace_to_sspace(ctx, Point::new(1.0, 1.0)),
      ..Default::default()
    };
  }
}


#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_no_room_collisions() {
    let mut l = Level::new();
    while l.gen_stage < LevelGenStage::PlaceObstacles {
      l.tick_level_gen();
    }
    l.tmp_collw.update();
    assert!(l.tmp_collw.contacts().next().is_none())
  }
}
