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

use ggez::{conf, ContextBuilder, event, graphics};
use world::render::WorldRender;
use world::World;

mod agents;
mod collision;
mod dungeongen;
mod util;
mod world;

fn main() {
  let cb = ContextBuilder::new("rougelike", "ggez")
    .window_setup(conf::WindowSetup::default()
                    .title("Rougelike!")
                  // TODO: Enable this and implement a fixed-ratio black bars solution or something
                  //.resizable(true)
    )
    .window_mode(conf::WindowMode::default()
      .dimensions(1600, 900)
    );

  let ctx = &mut cb.build().unwrap();

  let mut world = World::new();
  let mut renderer = WorldRender::new(&mut world, ctx);

  graphics::set_background_color(ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));
  event::run(ctx, &mut renderer).unwrap();
}
