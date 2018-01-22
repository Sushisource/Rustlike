extern crate ggez;

use ggez::{ContextBuilder, graphics, conf, event};

mod dungeongen;
mod util;
mod agents;

use dungeongen::Level;
use dungeongen::level_renderer::LevelRenderer;

fn main() {
  let cb = ContextBuilder::new("rougelike", "ggez")
    .window_setup(conf::WindowSetup::default()
      .title("Rougelike!")
    )
    .window_mode(conf::WindowMode::default()
      .dimensions(1365, 768)
    );

  let ctx = &mut cb.build().unwrap();

  let mut level = Level::new();
  let mut level_render = LevelRenderer::new(&mut level, ctx);

  graphics::set_background_color(ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));
  event::run(ctx, &mut level_render).unwrap();
}
