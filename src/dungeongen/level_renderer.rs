extern crate nalgebra;
extern crate ggez;
extern crate time;

use std::time::Duration;
use self::ggez::{Context, GameResult};
use self::ggez::event;
use self::ggez::event::{Keycode, Mod};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawMode, DrawParam, Drawable, FilterMode,
                           Image, Point, Rect};
use self::ggez::timer;

use dungeongen::{CellGrid, Level, CA_H, CA_W};

const CA_RENDERSCALE: f32 = 2.0;

pub struct LevelRenderer<'a> {
  level: &'a mut Level,
  fastmode: bool,
}

impl<'a> event::EventHandler for LevelRenderer<'a> {
  fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
    const DESIRED_FPS: u64 = 60;
    if !timer::check_update_time(ctx, DESIRED_FPS) {
      return Ok(());
    }
    // Tick the simulation
    if !self.level.level_gen_finished {
      let i = if self.fastmode { 15 } else { 2 };
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
      // let cave_ca = cave_points(&self.level.ca_grid);
      let cave_bounds = boundary_points(&self.level.ca_boundary);

      graphics::set_point_size(ctx, 0.008);
      graphics::set_line_width(ctx, 0.01);
      let ca_img_a = cave_ca_img(&self.level.ca_grid);
      let mut img =
        Image::from_rgba8(ctx, CA_W as u16, CA_H as u16, &ca_img_a)?;
      let params = DrawParam {
        scale: Point::new(
          CA_RENDERSCALE / CA_W as f32,
          CA_RENDERSCALE / CA_H as f32,
        ),
        ..Default::default()
      };
      // Don't make my pixels all blurry
      img.set_filter(FilterMode::Nearest);
      img.draw_ex(ctx, params)?;
      if !cave_bounds.is_empty() {
        graphics::line(ctx, cave_bounds.as_slice())?;
      }
    } else {
      let cave_bounds = boundary_points(&self.level.ca_boundary);
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      graphics::polygon(ctx, DrawMode::Fill, cave_bounds.as_slice())?;

      if self.level.rooms.len() > 0 {
        graphics::set_color(ctx, Color::new(0.2, 0.2, 0.2, 1.0))?;
        for room in &self.level.rooms {
          // TODO: Make rooms "drawable"
          // TODO: Scaling is all wrong
          // println!("{:?}", room);
          let mut r: Rect = room.into();
          r.x /= 200.0;
          r.y /= 200.0;
          r.w /= 200.0;
          r.h /= 200.0;
          graphics::rectangle(ctx, DrawMode::Fill, r)?;
        }
      }
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
  fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
    match keycode {
      Keycode::Space => {
        self.stop_render();
      }
      Keycode::Plus | Keycode::KpPlus => {
        self.fastmode = true;
      }
      _ => (), // Do nothing
    }
  }
}

impl<'a> LevelRenderer<'a> {
  pub fn new(level: &'a mut Level) -> LevelRenderer<'a> {
    LevelRenderer {
      level: level,
      fastmode: false,
    }
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }
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

/// Converts cellular automata space to unit space
fn ca_to_uspace(x: usize, y: usize) -> Point {
  let xp = (x as f32) / (CA_W as f32) - 0.5;
  let yp = (y as f32) / (CA_H as f32) - 0.5;
  Point::new(xp * CA_RENDERSCALE, yp * CA_RENDERSCALE)
}

fn boundary_points(boundary: &Vec<(i32, i32)>) -> Vec<Point> {
  boundary
    .iter()
    .map(|&(x, y)| ca_to_uspace(x as usize, y as usize))
    .collect::<Vec<Point>>()
}
