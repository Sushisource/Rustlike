extern crate nalgebra;
extern crate ggez;
extern crate time;

use std::time::Duration;
use self::ggez::{Context, GameResult};
use self::ggez::event;
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawMode, Point};
use self::ggez::timer;

use dungeongen::{CavePoints, CellGrid, Level, CA_H, CA_W};

pub struct LevelRenderer<'a> {
  level: &'a mut Level,
}

impl<'a> event::EventHandler for LevelRenderer<'a> {
  fn update(&mut self, _: &mut Context, _dt: Duration) -> GameResult<()> {
    // const DESIRED_FPS: u64 = 60;
    // if !timer::check_update_time(ctx, DESIRED_FPS) {
    //     return Ok(());
    // }
    // let seconds = 1.0 / (DESIRED_FPS as f64);
    // Tick the simulation
    if !self.level.level_gen_finished {
      self.level.tick_level_gen();
    }
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx);

    // In the first 4 stages we draw the CA evolution and the boundary
    if self.level.gen_stage <= 3 {
      let cave_ca = cave_points(&self.level.ca_grid);
      let cave_bounds = boundary_points(&self.level.ca_boundary);

      // TODO: Drawing these points becomes quite slow towards the end.
      // Potentially should just bake onto a texture
      graphics::set_point_size(ctx, 0.008);
      graphics::set_line_width(ctx, 0.01);
      graphics::points(ctx, cave_ca.as_slice())?;
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
          println!("{:?}", room);
          graphics::rectangle(ctx, DrawMode::Fill, room.into())?;
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
}

impl<'a> LevelRenderer<'a> {
  pub fn new(level: &'a mut Level) -> LevelRenderer<'a> {
    LevelRenderer { level: level }
  }

  pub fn stop_render(&mut self) -> () {
    self.level.level_gen_finished = true
  }
}

fn cave_points(ca_grid: &CellGrid) -> Vec<Point> {
  let mut points = cave_from_grid(ca_grid);
  let vlen = points.len();
  points[vlen - 1] = ca_to_uspace(0, 0).into();
  points[vlen - 2] = ca_to_uspace(0, CA_H).into();
  points[vlen - 3] = ca_to_uspace(CA_W, 0).into();
  points[vlen - 4] = ca_to_uspace(CA_W, CA_H).into();
  points
}

fn cave_from_grid(ca_grid: &CellGrid) -> CavePoints {
  let mut as_points: Vec<Point> = Vec::with_capacity(CA_W * CA_H);
  for x in 0..(CA_W - 1) {
    for y in 0..(CA_H - 1) {
      if ca_grid[x][y] {
        as_points.push(ca_to_uspace(x, y));
      }
    }
  }
  as_points
}

fn boundary_points(boundary: &Vec<(i32, i32)>) -> Vec<Point> {
  boundary
    .iter()
    .map(|&(x, y)| ca_to_uspace(x as usize, y as usize))
    .collect::<Vec<Point>>()
}

/// Converts cellular automata space to unit space
fn ca_to_uspace(x: usize, y: usize) -> Point {
  let xp = (x as f32) / (CA_W as f32) - 0.5;
  let yp = (y as f32) / (CA_H as f32) - 0.5;
  Point::new(xp * 1.9, yp * 1.9)
}
