extern crate ggez;

use self::ggez::{Context, GameResult};
use self::ggez::graphics;
use self::ggez::graphics::{DrawParam, Drawable};
use super::level_renderer::DrawablePt;

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
    let mut sim = CASim::new(128, 128, 20.0);
    sim.generate();
    Blobstacle { position: pos, sim }
  }
}

impl Drawable for Blobstacle {
  fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
    // TODO: Draw properly centered (looks like it's using dest as upper-right)
    let d = DrawablePt(self.position) * param.scale.into();
    let repositioned = DrawParam { dest: d.into(), ..param };
    graphics::draw_ex(ctx, &self.sim, repositioned)?;
    Ok(())
  }
}
