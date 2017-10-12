extern crate ggez;

use ggez::{Context, graphics, conf, event};

mod dungeongen;
mod util;

use dungeongen::Level;
use dungeongen::level_renderer::LevelRenderer;

fn main() {
  let mut c = conf::Conf::new();
  c.window_title = "Rougelike!".to_string();
  c.window_width = 1365;
  c.window_height = 768;

  let ctx = &mut Context::load_from_conf("roguelike", "ggez", c).unwrap();

  let mut level = Level::new();
  let mut level_render = LevelRenderer::new(&mut level, ctx);

  graphics::set_background_color(ctx, graphics::Color::new(0.0, 0.0, 0.0, 1.0));
  event::run(ctx, &mut level_render).unwrap();
}
