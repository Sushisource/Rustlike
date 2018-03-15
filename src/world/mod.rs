use agents::player::Player;
use dungeongen::Level;
use util::drawablept::DrawablePt;

pub mod render;

/// The entire world. Contains all world objects, and handles interaction
/// between subsystems.
pub struct World {
  level: Level,
  player: Player,
}

impl World {
  pub fn new() -> World {
    let level = Level::new();
    let player = Player::new(DrawablePt(level.middle()).into());
    World { level: level.into(), player }
  }

  // Should probably actually return set of things
  pub fn collide_check(&self) -> bool {
    false
  }
}
