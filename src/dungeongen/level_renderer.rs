extern crate ggez;
extern crate nalgebra;
extern crate rand;
extern crate time;

use std::time::Duration;
use self::ggez::{Context, GameResult};
use self::ggez::event;
use self::ggez::event::{Keycode, Mod};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawMode, DrawParam, Drawable, FilterMode,
                           Image, Point};
use self::ggez::timer;

use dungeongen;
use dungeongen::{CellGrid, Level, CA_H, CA_W, ca_to_uspace};

const CA_RENDERSCALE: f32 = 1.0;
type LevelPoint = dungeongen::Point;

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

    // In the first 4 stages we draw the CA evolution and the boundary
    if self.level.gen_stage <= 3 {
      let ca_img_a = cave_ca_img(&self.level.ca_grid);
      let mut img =
        Image::from_rgba8(ctx, CA_W as u16, CA_H as u16, &ca_img_a)?;
      let params = DrawParam { scale: Point::new(
        CA_RENDERSCALE / CA_W as f32 * self.screen_x,
        CA_RENDERSCALE / CA_H as f32 * self.screen_y,
      ),
                               dest: self.middle(),
                               ..Default::default() };
      // Don't make my pixels all blurry
      img.set_filter(FilterMode::Nearest);
      img.draw_ex(ctx, params)?;
      // Boundary drawing
      let cave_bounds = self.boundary_points(&self.level.ca_boundary);
      if !cave_bounds.is_empty() {
        graphics::set_line_width(ctx, 4.0);
        graphics::line(ctx, cave_bounds.as_slice())?;
      }
    } else {
      let cave_bounds = self.boundary_points(&self.level.ca_boundary);
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      graphics::polygon(ctx, DrawMode::Fill, cave_bounds.as_slice())?;

      if self.level.rooms.len() > 0 {
        for room in &self.level.rooms {
          let grayval = 0.2;
          graphics::set_color(ctx, Color::new(grayval, grayval, grayval, 1.0))?;
          let rd = self.room_to_sspace(room.center);
//          println!("{:?}", room);
//          println!("{:?}", rd);
          let drawps =
            DrawParam { dest: rd,
                        scale: self.lspace_to_sspace(LevelPoint::new(1.0, 1.0)),
                        ..Default::default() };
          room.draw_ex(ctx, drawps)?;
        }
      }
    }

    // TODO : Remove is test
    graphics::set_color(ctx, Color::new(1.0, 1.0, 1.0, 1.0))?;
    let player_d = self.lspace_to_sspace(self.level.middle());
    let drawps = DrawParam { dest: player_d,
                             ..Default::default() };
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
    LevelRenderer { level,
                    fastmode: false,
                    screen_x: ctx.conf.window_width as f32,
                    screen_y: ctx.conf.window_height as f32,
                    assets,
                    player: p, }
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }

  fn boundary_points(&self, boundary: &Vec<(i32, i32)>) -> Vec<Point> {
    boundary.iter()
            .map(|&(x, y)| {
              // TODO: This is ugly
              self.lspace_to_sspace(self.level.uspace_to_wspace(
                ca_to_uspace(x, y)),
              )
            })
            .collect::<Vec<Point>>()
  }

  fn room_to_sspace(&self, p: Point) -> Point {
    self.lspace_to_sspace(LevelPoint::new(p.x, p.y))
  }
  fn lspace_to_sspace(&self, p: LevelPoint) -> Point {
    let p = self.level.wspace_to_uspace(p);
    let sx = self.screen_x;
    let sy = self.screen_y;
    Point::new((p.x() * sx).ceil(), (p.y() * sy).ceil())
  }

  fn middle(&self) -> Point {
    Point::new(self.screen_x / 2.0, self.screen_y / 2.0)
  }
}

/// Converts the cave CA sim to a 1d array of RGBA 8 bit values
fn cave_ca_img(cell_grid: &CellGrid) -> [u8; CA_W * CA_H * 4] {
  let mut img = [0; CA_W * CA_H * 4];
  for x in 0..(CA_W - 1) {
    for y in 0..(CA_H - 1) {
      if cell_grid[x][y] {
        let i = (CA_W * y + x) * 4;
        img[i] = 0xAF;
        img[i + 1] = 0xAF;
        img[i + 2] = 0xAF;
        img[i + 3] = 0xFF;
      }
    }
  }
  img
}
