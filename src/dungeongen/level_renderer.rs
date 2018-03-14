extern crate ggez;
extern crate rand;
extern crate time;

use self::ggez::{Context, GameResult};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawParam, Drawable, Point2};

use dungeongen;
use dungeongen::Level;
use dungeongen::rooms::Room;
use util::context_help::ContextHelp;

pub type LevelPoint = dungeongen::Point;

impl Level {
  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    graphics::set_transform(ctx, DrawParam::default().into_matrix());
    graphics::apply_transformations(ctx)?;
    let sscale = ctx.sscale();
    let center_scale = self.l_center_scale(ctx);

    // In the stage 0 we draw the CA evolution and the boundary
    if self.gen_stage == 0 {
      &self.cave_sim.draw_evolution(ctx, sscale);
    } else {
      // Next stage, we render the cave as a polygon and place rooms
      graphics::set_color(ctx, Color::new(0.5, 0.5, 0.5, 1.0))?;
      self.cave_sim.draw_ex(ctx, sscale)?;

      graphics::set_transform(ctx, center_scale.into_matrix());
      graphics::apply_transformations(ctx)?;

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
      let croom = Room::new(self.middle(), 1.0, 1.0);
      graphics::set_color(ctx, Color::new(0.0, 0.5, 0.0, 1.0))?;
      croom.draw(ctx)?;
    }
    Ok(())
  }

  fn lspace_to_sspace(&self, ctx: &Context, p: LevelPoint) -> Point2 {
    let p = self.lspace_to_uspace(p);
    ctx.uspace_to_sspace(p)
  }

  pub fn l_center_scale(&self, ctx: &Context) -> DrawParam {
    return DrawParam {
      scale: self.lspace_to_sspace(ctx, LevelPoint::new(1.0, 1.0)),
      offset: Point2::new(0.5, 0.5),
      ..Default::default()
    };
  }
}
