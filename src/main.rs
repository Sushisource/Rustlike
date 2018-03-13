extern crate ggez;

use ggez::{ContextBuilder, graphics, conf, event};

mod dungeongen;
mod util;
mod agents;
mod world;

use world::{World};
use world::render::WorldRender;

fn main() {
  let cb = ContextBuilder::new("rougelike", "ggez")
    .window_setup(conf::WindowSetup::default()
      .title("Rougelike!")
    )
    .window_mode(conf::WindowMode::default()
      .dimensions(1365, 768)
    );

  let ctx = &mut cb.build().unwrap();

  let mut world = World::new();
  let mut renderer = WorldRender::new(&mut world, ctx);

  graphics::set_background_color(ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));
  event::run(ctx, &mut renderer).unwrap();
}
