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

use crate::{dungeongen::dungeongen, world::World};
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

  //let world = World::new();
  App::build()
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup.system())
    .add_startup_system(dungeongen.system())
    .run();
}

pub fn setup(commands: &mut Commands, mut state: ResMut<world::World>) {
  commands.spawn(Camera2dBundle::default()).spawn(CameraUiBundle::default());
}
