extern crate glium;

use std::fmt;
use dungeongen::Level;
use glium::{Surface, VertexBuffer, IndexBuffer, DrawParameters, PolygonMode,
            Program};
use glium::index::{NoIndices};
use glium::backend::Facade;
use glium::uniforms::Uniforms;


#[derive(Copy, Clone)]
pub struct Vertex {
  pub pos: [f32; 2],
}

impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Vert: {:?}", self.pos)
  }
}
implement_vertex!(Vertex, pos);

const NO_IXS: NoIndices = NoIndices(glium::index::PrimitiveType::Points);
static VERT_SHAD_DEF: &'static str = r#"
        #version 140
        in vec2 pos;
        void main() {
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;
static FRAG_CA: &'static str = r#"
        #version 140
        in vec4 gl_FragCoord;
        uniform vec2 resolution;
        out vec4 color;
        void main() {
            color = vec4(0.47, 0.59, 0.66, 1.0);
        }
    "#;
static FRAG_BOUNDS: &'static str = r#"
        #version 140
        in vec4 gl_FragCoord;
        uniform vec2 resolution;
        out vec4 color;
        void main() {
            color = vec4(0.22, 0.82, 0.71, 1.0);
        }
    "#;

pub struct LevelRenderer<'a> {
  level: &'a mut Level,
  cave_ca_vertb: VertexBuffer<Vertex>,
  cave_bounds_vertb: VertexBuffer<Vertex>,
  cave_bounds_indxb: IndexBuffer<u16>,
  ca_params: DrawParameters<'a>,
  bounds_params: DrawParameters<'a>,
  ca_prog: Program,
  bounds_prog: Program,
}

impl<'a> LevelRenderer<'a> {
  pub fn new<F>(level: &'a mut Level, display: &F) -> LevelRenderer<'a>
    where F: Facade {
    let ccv = {
      VertexBuffer::dynamic(display, &level.cave_verts()).unwrap()
    };
    let cbv = {
      VertexBuffer::dynamic(display, &level.boundary_verts()).unwrap()
    };
    let cbi = {
      IndexBuffer::dynamic(display, glium::index::PrimitiveType::LineStrip,
                           level.boundary_ix().as_slice()).unwrap()
    };

    LevelRenderer {
      level: level,
      cave_ca_vertb: ccv,
      cave_bounds_vertb: cbv,
      cave_bounds_indxb: cbi,

      ca_params: DrawParameters {
        point_size: Some(3.0),
        ..Default::default()
      },
      bounds_params: DrawParameters {
        line_width: Some(3.0),
        polygon_mode: PolygonMode::Line,
        ..Default::default()
      },

      ca_prog: Program::from_source(display, VERT_SHAD_DEF, FRAG_CA,
                                    None).unwrap(),
      bounds_prog: Program::from_source(display, VERT_SHAD_DEF, FRAG_BOUNDS,
                                        None).unwrap(),
    }
  }

  pub fn render_level_frame<S, U>(&mut self, frame: &mut S, uniforms: U) -> ()
    where S: Surface, U: Uniforms {
    // TODO: Stop drawing this after we're done with it
    frame.draw(&self.cave_ca_vertb, &NO_IXS, &self.ca_prog, &uniforms,
               &self.ca_params).unwrap();
    frame.draw(&self.cave_bounds_vertb, &self.cave_bounds_indxb,
               &self.bounds_prog, &uniforms, &self.bounds_params).unwrap();

    if !self.level.level_gen_finished {
      self.level.tick_level_gen();
    }
    let cave_ca = self.level.cave_verts();
    self.cave_ca_vertb.write(&cave_ca);
    let cave_bounds = self.level.boundary_verts();
    self.cave_bounds_vertb.write(&cave_bounds);
    self.cave_bounds_indxb.write(self.level.boundary_ix().as_slice());
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }
}
