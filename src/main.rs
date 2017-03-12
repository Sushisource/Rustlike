#[macro_use]
extern crate glium;

mod dungeongen;

use std::{thread, time};
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

  let frame_ratelimit = time::Duration::from_millis(5);

  println!("GL Version: {:?}", display.get_context().get_opengl_version());
  loop {
    let frame_start = time::Instant::now();

    let mut target = display.draw();
    target.clear_color_srgb(0.1, 0.2, 0.27, 0.0);
    let winsiz = display.get_context().get_framebuffer_dimensions();
    level_render.render_level_frame(&mut target, winsiz);
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

    let elapsed = frame_start.elapsed();
    if elapsed < frame_ratelimit {
      let wait = frame_ratelimit - elapsed;
      thread::sleep(wait);
    }
  }
}
