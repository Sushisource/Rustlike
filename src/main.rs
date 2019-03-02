#[macro_use]
extern crate derive_new;
extern crate ggez;
extern crate nalgebra as na;
extern crate ncollide2d as nc;
extern crate num;
#[macro_use]
extern crate num_derive;
extern crate rand;
#[macro_use]
extern crate roguelike_derive;
extern crate core;
#[macro_use]
extern crate log;
extern crate env_logger;

use crate::world::render::WorldRender;
use crate::world::World;
use env_logger::{Builder, Env};
use ggez::{conf, event, ContextBuilder};

mod agents;
mod collision;
mod dungeongen;
mod util;
mod world;

fn main() {
  // Set default log level to warn for everything, and info for our code
  Builder::from_env(Env::default().default_filter_or("warn,rustlike=info")).init();

  let cb = ContextBuilder::new("rougelike", "ggez")
    .window_setup(
      // TODO: Enable this and implement a fixed-ratio black bars solution or something
      //.resizable(true)
      conf::WindowSetup::default().title("Rougelike!"),
    )
    .window_mode(conf::WindowMode::default().dimensions(1600.0, 900.0));

  let (mut ctx, mut eloop) = cb.build().unwrap();

  let mut world = World::new();
  let mut renderer = WorldRender::new(&mut world, &mut ctx);
  // TODO: State is new in latest GGEZ, what do I use it for?
  event::run(&mut ctx, &mut eloop, &mut renderer).unwrap();
}
