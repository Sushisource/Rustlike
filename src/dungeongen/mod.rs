pub mod direction;
pub mod level;

mod blobstacle;
mod ca_simulator;
mod compound_room;
mod rooms;

use crate::{dungeongen::level::Level, world};
use bevy::prelude::*;

/// The startup system for dungeon generation.
pub fn dungeongen(commands: &mut Commands) {
  commands.insert_resource(world::World::new());
  // let level = Level::new();
  // commands.spawn(level);
}
