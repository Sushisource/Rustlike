#[macro_use]
extern crate glium;

mod dungeongen;

use glium::glutin::ElementState::Released;
use glium::glutin::{VirtualKeyCode, Event};
use glium::{DisplayBuild, Surface};
use glium::backend::Facade;

use dungeongen::{Level};
use dungeongen::level_renderer::LevelRenderer;

fn main() {
  let display = glium::glutin::WindowBuilder::new()
    .with_title("Dungeon game name")
    .with_dimensions(1024, 768)
    .build_glium().unwrap();

  let mut level = Level::new();
  let mut level_render = LevelRenderer::new(&mut level, &display);

  println!("GL Version: {:?}", display.get_context().get_opengl_version());
  loop {
    let winsiz = display.get_context().get_framebuffer_dimensions();
    let uniforms = glium::uniforms::UniformsStorage::new("resolution",
                                                         [winsiz.0 as f32,
                                                           winsiz.1 as f32]);

    let mut target = display.draw();
    target.clear_color_srgb(0.1, 0.2, 0.27, 0.0);
    level_render.render_level_frame(&mut target, uniforms);
    target.finish().unwrap();

    for ev in display.poll_events() {
      match ev {
        Event::Closed => return,
        Event::KeyboardInput(Released, _, Some(VirtualKeyCode::Space)) => {
          level_render.stop_render();
        },
        _ => (),
      }
    }
  }
}
