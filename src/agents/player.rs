extern crate ggez;

use self::ggez::{Context, GameResult};
use self::ggez::graphics;
use self::ggez::graphics::{DrawParam, Point2};

use super::Agent;
use super::super::util::{Assets, Point};
use super::super::dungeongen::level_renderer::DrawablePt;

static PLAYER_SYM: &'static str = "@";

pub struct Player {
  pos: Point,
}

impl Player {
  pub fn new(pos: Point) -> Player { Player { pos } }

  /// Currently we have to pass scale in here separately b/c we don't want
  /// the overal transform to scale our text, since we handle that with
  /// font sizes.
  pub fn draw(&self, ctx: &mut Context, assets: &mut Assets,
              scale: Point2) -> GameResult<()> {
    let d = DrawablePt(self.pos) * scale;
    let repositioned = DrawParam {
      dest: d.into(),
      offset: Point2::new(0.5, 0.5),
      ..DrawParam::default()
    };
    let txt = assets.txt(self, ctx);
    graphics::draw_ex(ctx, txt, repositioned)
  }
}

impl Agent for Player {
  fn width(&self) -> u32 { 1 }
  fn height(&self) -> u32 { 1 }
  fn symbol(&self) -> &'static str { PLAYER_SYM }
  fn pos(&self) -> Point { self.pos }
}
