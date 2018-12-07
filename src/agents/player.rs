extern crate ggez;
extern crate nalgebra;

use super::Agent;
use ggez::graphics;
use ggez::graphics::{DrawParam, Point2, Vector2};
use ggez::{Context, GameResult};
use crate::util::Assets;

static PLAYER_SYM: &'static str = "@";

pub struct Player {
  pos: Point2,
}

impl Player {
  pub fn new(pos: Point2) -> Player {
    Player { pos }
  }

  /// Currently we have to pass scale in here separately b/c we don't want
  /// the overal transform to scale our text, since we handle that with
  /// font sizes.
  pub fn draw(&self, ctx: &mut Context, assets: &mut Assets, scale: Point2) -> GameResult<()> {
    let d = Point2::new(self.pos.x * scale.x, self.pos.y * scale.y);
    let repositioned = DrawParam {
      dest: d,
      // This offset is because the draw point is the upper-left corner of
      // the text.
      offset: Point2::new(0.60, 0.60),
      ..DrawParam::default()
    };
    let txt = assets.agent_txt(self, ctx);
    graphics::draw_ex(ctx, txt, repositioned)
  }
}

impl Agent for Player {
  fn width(&self) -> u32 {
    1
  }
  fn height(&self) -> u32 {
    1
  }
  fn symbol(&self) -> &'static str {
    PLAYER_SYM
  }
  fn pos(&self) -> Point2 {
    self.pos
  }

  fn trans(&mut self, by: Vector2) {
    self.pos += by;
  }
}
