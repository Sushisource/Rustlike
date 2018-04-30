extern crate ggez;

use ggez::{Context, GameResult};
use ggez::graphics::DrawParam;
use super::ca_simulator::CASim;
use util::Point;

/// Blobstacles are backed by a CA sim but have additional information like
/// a position, ability to determine intersections, etc.
pub struct Blobstacle {
  position: Point,
  sim: CASim,
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
    let repositioned = DrawParam { dest: self.position, ..DrawParam::default() };
    self.sim.draw(ctx, repositioned)
  }
}
