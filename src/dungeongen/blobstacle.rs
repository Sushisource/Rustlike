use super::ca_simulator::CASim;
use super::Point;

/// Blobstacles are backed by a CA sim but have additional information like
/// a position, ability to determine intersections, etc.
pub struct Blobstacle {
  position: Point,
  sim: CASim
}

impl Blobstacle {
  pub fn new() -> Blobstacle {
    Blobstacle { position: Point::new(0.0, 0.0), sim: CASim::new(0.1) }
  }
}