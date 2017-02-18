extern crate glium;
extern crate nalgebra as na;

use std::fmt;
use dungeongen::{Level, CavePoints, CA_H, CA_BUFSIZ, CA_W, CellGrid};
use glium::{Surface, VertexBuffer, IndexBuffer, DrawParameters, PolygonMode,
            Program};
use glium::index::{NoIndices};
use glium::backend::Facade;
use glium::uniforms::Uniforms;
use self::na::Vector2;

pub type Point = Vector2<f32>;

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
      VertexBuffer::dynamic(display, cave_verts(&level.ca_grid).as_ref())
        .unwrap()
    };
    let cbv = {
      VertexBuffer::dynamic(display, boundary_verts(&level.boundary).as_ref())
        .unwrap()
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
    let cave_ca = cave_verts(&self.level.ca_grid);
    self.cave_ca_vertb.write(&cave_ca);
    let cave_bounds = boundary_verts(&self.level.boundary);
    self.cave_bounds_vertb.write(&cave_bounds);
    self.cave_bounds_indxb.write(self.level.boundary_ix().as_slice());
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }
}

fn cave_verts(ca_grid: &CellGrid) -> Vec<Vertex> {
  let cavep = cave_from_grid(ca_grid);
  let mut verts = cavep.iter().map(|&x| Vertex { pos: [x.x, x.y] })
    .collect::<Vec<Vertex>>();
  // We have to pad the array so it's always the same size, so openGL doesn't
  // freak out when we update it with more or less verticies
  for _ in verts.len()..CA_BUFSIZ {
    // We're just putting them way off in the corner somewhere invisible
    verts.push(Vertex { pos: [-10.0, -10.0] });
  }
  let vlen = verts.len();
  verts[vlen - 1] = Vertex { pos: *project_to_unitspace(0, 0).as_ref() };
  verts[vlen - 2] = Vertex { pos: *project_to_unitspace(0, CA_H).as_ref() };
  verts[vlen - 3] = Vertex { pos: *project_to_unitspace(CA_W, 0).as_ref() };
  verts[vlen - 4] = Vertex { pos: *project_to_unitspace(CA_W, CA_H).as_ref() };
  verts
}

fn cave_from_grid(ca_grid: &CellGrid) -> CavePoints {
  let mut as_points: Vec<Point> = Vec::with_capacity(CA_W * CA_H);
  for x in 0..(CA_W - 1) {
    for y in 0..(CA_H - 1) {
      if ca_grid[x][y] {
        as_points.push(project_to_unitspace(x, y));
      }
    }
  }
  as_points
}

fn boundary_verts(boundary: &Vec<(i32, i32)>) -> Vec<Vertex> {
  let mut verts = boundary.iter().map(|&(x, y)| {
    let as_pt = project_to_unitspace(x as usize, y as usize);
    Vertex { pos: *as_pt.as_ref() }
  }).collect::<Vec<Vertex>>();
  for _ in verts.len()..CA_BUFSIZ {
    // We're just putting them way off in the corner somewhere invisible
    verts.push(Vertex { pos: [-10.0, -10.0] });
  }
  verts
}

fn project_to_unitspace(x: usize, y: usize) -> Point {
  let xp = (x as f32) / (CA_W as f32) - 0.5;
  let yp = (y as f32) / (CA_H as f32) - 0.5;
  Point::new(xp * 1.5, yp * 1.5)
}
