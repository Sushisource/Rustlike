extern crate ggez;

use self::ggez::{Context, GameResult};
use self::ggez::graphics::DrawParam;

use util::drawablept::DrawablePt;
use super::ca_simulator::CASim;
use super::Point;

/// Blobstacles are backed by a CA sim but have additional information like
/// a position, ability to determine intersections, etc.
pub struct Blobstacle {
  position: Point,
  sim: CASim
}

impl Blobstacle {
  pub fn new(pos: Point) -> Blobstacle {
    // TODO: I think this number is effectively "blob width in world units"
    // But need to verify that.
    let mut sim = CASim::new(128, 128, 10.0);
    sim.generate();
    Blobstacle { position: pos, sim }
  }

  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    let d = DrawablePt(self.position);
    let repositioned = DrawParam { dest: d.into(), ..DrawParam::default() };
    self.sim.draw(ctx, repositioned)
  }
}
