extern crate ggez;
extern crate rand;
extern crate time;

use std;
use std::time::Duration;
use self::ggez::{Context, GameResult};
use self::ggez::event;
use self::ggez::event::{Keycode, Mod};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawParam, Drawable, Point2, Vector2};
use self::ggez::timer;

use dungeongen;
use dungeongen::Level;
use dungeongen::rooms::Room;
use dungeongen::geo::prelude::MapCoords;
use agents::player::Player;
use agents::Agent;
use util::Assets;

type LevelPoint = dungeongen::Point;

pub struct LevelRenderer<'a> {
  level: &'a mut Level,
  fastmode: bool,
  screen_x: f32,
  screen_y: f32,
  assets: Assets,
  player: Player,
}

impl<'a> event::EventHandler for LevelRenderer<'a> {
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
    const DESIRED_FPS: u32 = 60;
    if !timer::check_update_time(ctx, DESIRED_FPS) {
      return Ok(());
    }
    // Tick the simulation
    if !self.level.level_gen_finished {
      let i = if self.fastmode { 12 } else { 2 };
      for _ in 1..i {
        self.level.tick_level_gen();
      }
    }
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx);
    graphics::set_transform(ctx, DrawParam::default().into_matrix());
    graphics::apply_transformations(ctx)?;
    let scaler = DrawParam {
      scale: self.lscale(),
      offset: Point2::new(0.5, 0.5),
      ..Default::default()
    };

    // In the stage 0 we draw the CA evolution and the boundary
    if self.level.gen_stage == 0 {
      &self.level.cave_sim.draw_evolution(ctx, self.sscale());
    } else {
      // Next stage, we render the cave as a polygon and place rooms
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      self.level.cave_sim.draw_ex(ctx, self.sscale())?;

      graphics::set_transform(ctx, scaler.into_matrix());
      graphics::apply_transformations(ctx)?;

      if self.level.rooms.len() > 0 {
        for room in &self.level.rooms {
          let grayval = 0.2;
          graphics::set_color(ctx, Color::new(grayval, grayval, grayval, 1.0))?;
          room.draw(ctx)?;
        }
      }

      if self.level.obstacles.len() > 0 {
        for obstacle in &self.level.obstacles {
          graphics::set_color(ctx, (227, 77, 40).into())?;
          obstacle.draw(ctx)?;
        }
      }
      // Test center room of one sq unit
      let croom = Room::new(self.level.middle(), 1.0, 1.0);
      graphics::set_color(ctx, Color::new(0.0, 0.5, 0.0, 1.0))?;
      croom.draw(ctx)?;

      graphics::set_transform(ctx, DrawParam::default().into_matrix());
      graphics::apply_transformations(ctx)?;
      // TODO: Create "world" and move player obj into there
      graphics::set_color(ctx, Color::new(1.0, 1.0, 1.0, 1.0))?;
      self.player.draw(ctx, &mut self.assets, scaler.scale)?;
    }

    graphics::present(ctx);
    // And sleep for 0 seconds.
    // This tells the OS that we're done using the CPU but it should
    // get back to this program as soon as it can.
    // This prevents the game from using 100% CPU all the time
    // even if vsync is off.
    timer::sleep(Duration::from_secs(0));
    Ok(())
  }

  // Handle key events.  These just map keyboard events
  // and alter our input state appropriately.
  fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode,
                    _keymod: Mod, _repeat: bool) {
    match keycode {
      Keycode::Space => {
        self.stop_render();
      }
      Keycode::Plus | Keycode::KpPlus => {
        self.fastmode = !self.fastmode;
      }
      Keycode::Up => {
        self.player.trans(Vector2::new(0.0, -1.0));
      }
      Keycode::Down => {
        self.player.trans(Vector2::new(0.0, 1.0));
      }
      _ => (), // Do nothing
    }
  }
}

impl<'a> LevelRenderer<'a> {
  pub fn new(level: &'a mut Level, ctx: &mut Context) -> LevelRenderer<'a> {
    let assets = Assets::new(ctx);
    let player = Player::new(DrawablePt(level.middle()).into());
    LevelRenderer {
      level,
      fastmode: true,
      screen_x: ctx.conf.window_mode.width as f32,
      screen_y: ctx.conf.window_mode.height as f32,
      assets,
      player: player,
    }
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }

  fn uspace_to_sspace(&self, p: LevelPoint) -> Point2 {
    let sx = self.screen_x;
    let sy = self.screen_y;
    Point2::new(p.x() * sx, p.y() * sy)
  }
  fn lspace_to_sspace(&self, p: LevelPoint) -> Point2 {
    let p = self.level.wspace_to_uspace(p);
    self.uspace_to_sspace(p)
  }

  fn middle(&self) -> Point2 {
    Point2::new(self.screen_x / 2.0, self.screen_y / 2.0)
  }

  fn sscale(&self) -> DrawParam {
    DrawParam {
      scale: Point2::new(self.screen_x, self.screen_y),
      dest: self.middle(),
      ..Default::default()
    }
  }

  fn lscale(&self) -> Point2 {
    self.lspace_to_sspace(LevelPoint::new(1.0, 1.0))
  }
}

// Sorta lame that we have to do this b/c can't implement traits for non-crate
// types
#[derive(Copy, Clone, Debug)]
pub struct DrawablePt(pub LevelPoint);

impl From<DrawablePt> for Point2 {
  fn from(dp: DrawablePt) -> Self {
    let DrawablePt(p) = dp;
    Point2::new(p.x(), p.y())
  }
}

impl From<Point2> for DrawablePt {
  fn from(dp: Point2) -> Self {
    DrawablePt(LevelPoint::new(dp.x, dp.y))
  }
}

impl std::ops::Mul for DrawablePt {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    let DrawablePt(p) = self;
    let DrawablePt(p2) = rhs;
    DrawablePt(LevelPoint::new(p.x() * p2.x(), p.y() * p2.y()))
  }
}

impl std::ops::Mul<Point2> for DrawablePt {
  type Output = Self;

  fn mul(self, rhs: Point2) -> Self {
    let DrawablePt(p) = self;
    DrawablePt(LevelPoint::new(p.x() * rhs.coords.x, p.y() * rhs.coords.y))
  }
}

impl DrawablePt {
  /// Truncates floating point numbers to avoid rendering artifacts
  pub fn snap(&self) -> Self {
    let DrawablePt(p) = *self;
    DrawablePt(p.map_coords(&|&(x, y)| (x.ceil(), y.ceil())))
  }
}
