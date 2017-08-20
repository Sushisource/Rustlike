extern crate ggez;

use ggez::conf;
use ggez::event;
use ggez::Context;
use ggez::graphics;

mod dungeongen;
mod util;

use dungeongen::Level;
use dungeongen::level_renderer::LevelRenderer;

fn main() {
  let mut c = conf::Conf::new();
  c.window_title = "Rougelike!".to_string();
  // TODO: Fix ratio stuff
  c.window_width = 768;
  c.window_height = 768;

  let ctx = &mut Context::load_from_conf("roguelike", "ggez", c).unwrap();

  let mut level = Level::new();
  let mut level_render = LevelRenderer::new(&mut level);
  
  graphics::set_screen_coordinates(ctx, -1.0, 1.0, -1.0, 1.0).unwrap();
  event::run(ctx, &mut level_render).unwrap();
}
