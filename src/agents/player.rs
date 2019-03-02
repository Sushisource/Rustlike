extern crate ggez;
extern crate nalgebra;

use super::Agent;
use crate::util::Assets;
use ggez::graphics;
use ggez::graphics::{DrawParam};
use ggez::{Context, GameResult};
use ggez::graphics::Color;
use crate::util::Vec2;
use crate::util::Point;

static PLAYER_SYM: &'static str = "@";

pub struct Player {
  pos: Point,
}

impl Player {
  pub fn new(pos: Point) -> Player {
    Player { pos }
  }

  /// Currently we have to pass scale in here separately b/c we don't want
  /// the overal transform to scale our text, since we handle that with
  /// font sizes.
  pub fn draw(&self, ctx: &mut Context, assets: &mut Assets, scale: Vec2) -> GameResult<()> {
    let d = Point::new(self.pos.x * scale.x, self.pos.y * scale.y);
    let repositioned = DrawParam {
      dest: d.into(),
      // This offset is because the draw point is the upper-left corner of
      // the text.
      offset: Point::new(0.60, 0.60).into(),
      color: Color::new(1.0, 1.0, 1.0, 1.0),
      ..DrawParam::default()
    };
    let txt = assets.agent_txt(self);
    graphics::draw(ctx, txt, repositioned)
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
  fn pos(&self) -> Point {
    self.pos
  }

  fn trans(&mut self, by: Vec2) {
    self.pos += by;
  }
}
