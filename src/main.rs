#[macro_use] extern crate glium;

mod dungeongen;

use glium::glutin::ElementState::Released;
use glium::glutin::VirtualKeyCode;
use glium::glutin::Event;

fn main() {
  use glium::{DisplayBuild, Surface};
  use glium::backend::Facade;
  use glium::draw_parameters::PolygonMode;
  use dungeongen::Level;

  let display = glium::glutin::WindowBuilder::new()
    .with_dimensions(800, 600)
    .build_glium().unwrap();


  let level = Level::gen_cave();
  let mut shape = level.cave_verts();

  let vertex_buffer = glium::VertexBuffer::dynamic(&display, &shape).unwrap();
  let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

  let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

  let fragment_shader_src = r#"
        #version 140
        in vec4 gl_FragCoord;
        uniform vec2 resolution;
        out vec4 color;
        void main() {
            color = vec4(1.0, 1.0, 0.5, 1.0);
        }
    "#;

  let draw_params = glium::DrawParameters {
    line_width: Some(3.0),
    polygon_mode: PolygonMode::Line,
    ..Default::default()
  };

  let program = glium::Program::from_source(&display,
                                            vertex_shader_src,
                                            fragment_shader_src,
                                            None).unwrap();

  loop {
    let winsiz = display.get_context().get_framebuffer_dimensions();
    let uniforms = glium::uniforms::UniformsStorage::new("resolution",
                                                         [winsiz.0 as f32,
                                                          winsiz.1 as f32]);

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    target.draw(&vertex_buffer, &indices, &program, &uniforms, &draw_params)
      .unwrap();
    target.finish().unwrap();

    for ev in display.poll_events() {
      match ev {
        Event::Closed => return,
        Event::KeyboardInput(Released, _, Some(VirtualKeyCode::Space)) => {
          shape = Level::gen_cave().cave_verts();
          vertex_buffer.write(&shape);
        },
        _ => (),
      }
    }
  }
}
