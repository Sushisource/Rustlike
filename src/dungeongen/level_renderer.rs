extern crate ggez;
extern crate rand;
extern crate time;

use self::ggez::{Context, GameResult};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawParam, Point2};

use dungeongen;
use dungeongen::Level;
use util::context_help::ContextHelp;

pub type LevelPoint = dungeongen::Point;

impl Level {
  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    graphics::set_transform(ctx, DrawParam::default().into_matrix());
    graphics::apply_transformations(ctx)?;
    let sscale = ctx.sscale();
    let center_scale = self.lscale(ctx);

    // In the stage 0 we draw the CA evolution and the boundary
    if self.gen_stage == 0 {
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
      // Test drawing cave bounding box
//      let bb = self.cave_bound_box();
//      graphics::set_color(ctx, Color::new(0.9, 0.0, 0.0, 0.2))?;
//      ctx.draw_bb(&bb)?;
      // Test center room of one sq unit
      graphics::set_color(ctx, Color::new(0.0, 0.5, 0.0, 1.0))?;
      ctx.center_rect(self.middle(), 1.0, 1.0)?;
    }
    Ok(())
  }

  fn lspace_to_sspace(&self, ctx: &Context, p: LevelPoint) -> Point2 {
    let p = self.lspace_to_uspace(p);
    ctx.uspace_to_sspace(p)
  }

  pub fn sspace_to_lspace(&self, ctx: &Context, p: Point2) -> Point2 {
    let p = ctx.sspace_to_uspace(p);
    self.uspace_to_lspace(p)
  }

  fn u_to_l_scale(&self) -> DrawParam {
    DrawParam {
      scale: self.uspace_to_lspace(Point2::new(1.0, 1.0)),
      ..Default::default()
    }
  }

  pub fn lscale(&self, ctx: &Context) -> DrawParam {
    return DrawParam {
      scale: self.lspace_to_sspace(ctx, LevelPoint::new(1.0, 1.0)),
      ..Default::default()
    };
  }
}
