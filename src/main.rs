#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate roguelike_derive;
#[macro_use]
extern crate log;

use crate::{
  dungeongen::{draw_ca_boundary, draw_evolution, dungeongen, dungeongen_init},
  world::World,
};
use bevy::prelude::*;
use env_logger::{Builder, Env};

mod agents;
mod collision;
mod dungeongen;
mod util;
mod world;

fn main() {
  // Set default log level to warn for everything, and info for our code
  Builder::from_env(Env::default().default_filter_or("warn,rustlike=info")).init();

  // TODO: Use state system to go from menu->generation->play etc.
  App::build()
    .add_resource(WindowDescriptor {
      title: "rustlike".to_string(),
      width: 1000.,
      height: 1000.,
      vsync: true,
      resizable: false,
      ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_resource(world::World::new())
    .add_startup_system(setup.system())
    .add_startup_system(dungeongen_init.system())
    .add_system(dungeongen.system())
    .add_system(draw_evolution.system())
    .add_system(draw_ca_boundary.system())
    .run();
}

pub fn setup(commands: &mut Commands) {
  commands.spawn(Camera2dBundle::default()).spawn(CameraUiBundle::default());
}
