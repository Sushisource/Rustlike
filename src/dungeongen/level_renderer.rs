extern crate ggez;
extern crate rand;
extern crate time;

use self::ggez::{Context, GameResult};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawParam, Drawable, Point2};

use dungeongen;
use dungeongen::Level;
use dungeongen::rooms::Room;

type LevelPoint = dungeongen::Point;

pub struct LevelRenderer<'a> {
  level: &'a Level,
  screen_x: f32,
  screen_y: f32,
}

impl<'a> LevelRenderer<'a> {
  pub fn new(level: &'a Level, ctx: &mut Context) -> LevelRenderer<'a> {
    LevelRenderer {
      level,
      screen_x: ctx.conf.window_mode.width as f32,
      screen_y: ctx.conf.window_mode.height as f32,
    }
  }

  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    graphics::set_transform(ctx, DrawParam::default().into_matrix());
    graphics::apply_transformations(ctx)?;

    // In the stage 0 we draw the CA evolution and the boundary
    if self.level.gen_stage == 0 {
      &self.level.cave_sim.draw_evolution(ctx, self.sscale());
    } else {
      // Next stage, we render the cave as a polygon and place rooms
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      self.level.cave_sim.draw_ex(ctx, self.sscale())?;

      graphics::set_transform(ctx, self.l_center_scale().into_matrix());
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
    }
    Ok(())
  }

  // TODO: Probably none of this should be in here.

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

  pub fn l_center_scale(&self) -> DrawParam {
    return DrawParam {
      scale: self.lscale(),
      offset: Point2::new(0.5, 0.5),
      ..Default::default()
    };
  }
}
