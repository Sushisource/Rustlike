use crate::util::Point;
use ggez::error::GameResult;
use ggez::graphics::{Color, DrawParam};
use ggez::graphics::{DrawMode, Mesh};
use ggez::{graphics, Context};

lazy_static! {
  static ref MOUSE_TARGET_COLOR: Color = Color::new(0.7, 1.0, 0.7, 0.5);
}

pub struct MouseTarget {
  mesh: Mesh,
}

impl MouseTarget {
  pub fn new(ctx: &mut Context) -> GameResult<Self> {
    Ok(MouseTarget {
      mesh: Mesh::new_circle(
        ctx,
        DrawMode::fill(),
        Point::new(0.0, 0.0),
        0.5,
        0.001,
        // TODO: Configureable target color?
        *MOUSE_TARGET_COLOR,
      )?,
    })
  }

  pub fn draw(&self, ctx: &mut Context, pos: Point, player_pos: Point) -> GameResult<()> {
    graphics::draw(ctx, &self.mesh, DrawParam::new().dest(pos))?;
    let line = Mesh::new_line(ctx, &[pos, player_pos], 0.3, *MOUSE_TARGET_COLOR)?;
    graphics::draw(ctx, &line, DrawParam::new())?;
    graphics::draw(ctx, &self.mesh, DrawParam::new().dest(player_pos))?;
    Ok(())
  }
}
