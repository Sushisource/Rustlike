pub mod direction;
pub mod level;

mod blobstacle;
mod ca_simulator;
mod compound_room;
mod rooms;

use crate::dungeongen::ca_simulator::CASim;
use crate::{dungeongen::level::Level, world};
use bevy::prelude::*;

struct TickLevelGen;

/// The system for dungeon generation.
pub fn dungeongen(
  commands: &mut Commands,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut state: ResMut<world::World>,
) {
  if !state.level.level_gen_finished {
    for _ in 1..10 {
      state.level.tick_level_gen();
    }
  }
  commands.spawn(SpriteBundle {
    sprite: Sprite::new(Vec2::new(10.0, 10.0)),
    transform: Transform::from_translation(Vec3::new(0.0, -0.0, 0.0)),
    material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
    ..Default::default()
  });
  state.level.cave_sim.draw_evolution(commands, materials);
}
