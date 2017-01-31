#[macro_use]
extern crate glium;

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
    .with_srgb(Some(true))
    .with_dimensions(800, 600)
    .build_glium().unwrap();

  let mut level = Level::gen_cave();
  let mut cave_ca = level.cave_verts();
  let mut cave_bounds = level.boundary_verts();

  let cave_ca_buff = glium::VertexBuffer::dynamic(&display, &cave_ca).unwrap();
  let cave_bounds_buff = glium::VertexBuffer::dynamic(&display,
                                                      &cave_bounds).unwrap();
  let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
  let indices2 = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);

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
            color = vec4(0.47, 0.59, 0.66, 1.0);
        }
    "#;
  let fragment_shader_2 = r#"
        #version 140
        in vec4 gl_FragCoord;
        uniform vec2 resolution;
        out vec4 color;
        void main() {
            color = vec4(0.22, 0.81, 0.70, 1.0);
        }
    "#;

  let draw_params = glium::DrawParameters {
    line_width: Some(3.0),
    point_size: Some(4.0),
    polygon_mode: PolygonMode::Line,
    ..Default::default()
  };

  let program = glium::Program::from_source(&display,
                                            vertex_shader_src,
                                            fragment_shader_src,
                                            None).unwrap();
  let program2 = glium::Program::from_source(&display,
                                             vertex_shader_src,
                                             fragment_shader_2,
                                             None).unwrap();

  loop {
    let winsiz = display.get_context().get_framebuffer_dimensions();
    let uniforms = glium::uniforms::UniformsStorage::new("resolution",
                                                         [winsiz.0 as f32,
                                                           winsiz.1 as f32]);

    let mut target = display.draw();
    target.clear_color_srgb(0.1, 0.2, 0.27, 0.0);
    target.draw(&cave_ca_buff, &indices, &program, &uniforms, &draw_params)
      .unwrap();
    target.draw(&cave_bounds_buff, &indices2, &program2, &uniforms, &draw_params)
      .unwrap();
    target.finish().unwrap();

    if !level.level_gen_finished {
      level.tick_level_gen();
    }
    cave_ca = level.cave_verts();
    cave_bounds = level.boundary_verts();
    cave_ca_buff.write(&cave_ca);
    cave_bounds_buff.write(&cave_bounds);

    for ev in display.poll_events() {
      match ev {
        Event::Closed => return,
        Event::KeyboardInput(Released, _, Some(VirtualKeyCode::Space)) => {
          level.level_gen_finished = true;
        },
        _ => (),
      }
    }
  }
}
