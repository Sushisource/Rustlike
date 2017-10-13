extern crate ggez;
extern crate nalgebra;
extern crate rand;
extern crate time;

use std;
use std::time::Duration;
use self::ggez::{Context, GameResult};
use self::ggez::event;
use self::ggez::event::{Keycode, Mod};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawMode, DrawParam, Drawable, Point};
use self::ggez::timer;

use dungeongen;
use dungeongen::Level;
use dungeongen::geo::prelude::MapCoords;

type LevelPoint = dungeongen::Point;

#[derive(Copy, Clone, Debug)]
pub struct DrawablePt(pub LevelPoint);

struct Assets {
  font: graphics::Font,
}

pub struct LevelRenderer<'a> {
  level: &'a mut Level,
  fastmode: bool,
  screen_x: f32,
  screen_y: f32,
  assets: Assets,
  player: graphics::Text,
}

impl<'a> event::EventHandler for LevelRenderer<'a> {
  fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
    const DESIRED_FPS: u64 = 60;
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

    // In the stage 0 we draw the CA evolution and the boundary
    if self.level.gen_stage == 0 {
      let params = DrawParam {
        scale: Point::new(self.screen_x, self.screen_y),
        dest: self.middle(),
        ..Default::default()
      };
      &self.level.cave_sim.draw_evolution(ctx, params);
    } else {
      // Next stage, we render the cave as a polygon and place rooms
      let cave_bounds = self.upts_to_sspace(
        &self.level.cave_sim.uspace_boundary());
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      graphics::polygon(ctx, DrawMode::Fill, cave_bounds.as_slice())?;

      if self.level.obstacles.len() > 0 {
        for obstacle in &self.level.obstacles {
          let boundary = self.upts_to_sspace(&obstacle.uspace_boundary());
          graphics::set_color(ctx, Color::new(0.5, 0.5, 0.8, 1.0))?;
          graphics::polygon(ctx, DrawMode::Fill, boundary.as_slice())?;
        }
      }

      if self.level.rooms.len() > 0 {
        for room in &self.level.rooms {
          let grayval = 0.2;
          graphics::set_color(ctx, Color::new(grayval, grayval, grayval, 1.0))?;
          let rd = self.room_to_sspace(room.center);
          let drawps =
            DrawParam {
              dest: rd,
              scale: self.lspace_to_sspace(LevelPoint::new(1.0, 1.0)),
              ..Default::default()
            };
          room.draw_ex(ctx, drawps)?;
        }
      }
    }

    // TODO: Remove is test
    graphics::set_color(ctx, Color::new(1.0, 1.0, 1.0, 1.0))?;
    let player_d = self.lspace_to_sspace(self.level.middle());
    let drawps = DrawParam {
      dest: player_d,
      ..Default::default()
    };
    graphics::draw_ex(ctx, &self.player, drawps)?;

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
  fn key_down_event(&mut self, keycode: Keycode,
                    _keymod: Mod, _repeat: bool) {
    match keycode {
      Keycode::Space => {
        self.stop_render();
      }
      Keycode::Plus | Keycode::KpPlus => {
        self.fastmode = !self.fastmode;
      }
      _ => (), // Do nothing
    }
  }
}

impl<'a> LevelRenderer<'a> {
  pub fn new(level: &'a mut Level, ctx: &mut Context) -> LevelRenderer<'a> {
    let font = graphics::Font::new(ctx, "/consola.ttf", 16).unwrap();
    let assets = Assets { font };
    let p = graphics::Text::new(ctx, "@", &assets.font).unwrap();
    LevelRenderer {
      level,
      fastmode: false,
      screen_x: ctx.conf.window_width as f32,
      screen_y: ctx.conf.window_height as f32,
      assets,
      player: p,
    }
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }

  // TODO: This needs to be moved?
  pub fn upts_to_sspace(&self, boundary: &Vec<LevelPoint>) -> Vec<Point> {
    boundary.iter().map(|&p| {
      self.uspace_to_sspace(p)
    }).collect::<Vec<Point>>()
  }

  fn room_to_sspace(&self, p: Point) -> Point {
    self.lspace_to_sspace(LevelPoint::new(p.x, p.y))
  }
  fn uspace_to_sspace(&self, p: LevelPoint) -> Point {
    let sx = self.screen_x;
    let sy = self.screen_y;
    Point::new(p.x() * sx, p.y() * sy)
  }
  fn lspace_to_sspace(&self, p: LevelPoint) -> Point {
    let p = self.level.wspace_to_uspace(p);
    self.uspace_to_sspace(p)
  }

  fn middle(&self) -> Point {
    Point::new(self.screen_x / 2.0, self.screen_y / 2.0)
  }
}

// Sorta lame that we have to do this b/c can't implement traits for non-crate
// types
impl From<DrawablePt> for Point {
  fn from(dp: DrawablePt) -> Self {
    let DrawablePt(p) = dp;
    Point::new(p.x(), p.y())
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

impl DrawablePt {
  /// Truncates floating point numbers to avoid rendering artifacts
  pub fn snap(&self) -> Self {
    let DrawablePt(p) = *self;
    DrawablePt(p.map_coords(&|&(x, y)| (x.ceil(), y.ceil())))
  }
}
