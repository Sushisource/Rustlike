extern crate glium;
extern crate nalgebra as na;
extern crate graphics;

use std::fmt;
use dungeongen::{Level, CavePoints, CA_H, CA_BUFSIZ, CA_W, CellGrid};
use glium::{Surface, VertexBuffer, IndexBuffer, DrawParameters, PolygonMode,
            Program};
use glium::index::{NoIndices, PrimitiveType};
use glium::backend::Facade;
use glium::uniforms::Uniforms;
use self::na::Vector2;
use super::polyfill::polyfill_calc;

pub type Point = Vector2<f32>;

#[derive(Copy, Clone)]
pub struct Vertex {
  pub pos: [f32; 2],
}

impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "V:{:?}", self.pos)
  }
}
implement_vertex!(Vertex, pos);

const NO_IXS: NoIndices = NoIndices(PrimitiveType::Points);
const NO_IXS_TRI: NoIndices = NoIndices(PrimitiveType::TrianglesList);
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

pub struct LevelRenderer<'a, F: 'a> where F: Facade {
  level: &'a mut Level,
  display: &'a F,
  render_stage: u8,
  cave_ca_vertb: VertexBuffer<Vertex>,
  cave_bounds_vertb: VertexBuffer<Vertex>,
  cave_bounds_indxb: IndexBuffer<u16>,
  ca_params: DrawParameters<'a>,
  bounds_params: DrawParameters<'a>,
  cave_params: DrawParameters<'a>,
  ca_prog: Program,
  bounds_prog: Program,
}

impl<'a, F: Facade> LevelRenderer<'a, F> {
  pub fn new(level: &'a mut Level, display: &'a F) -> LevelRenderer<'a, F>
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
      IndexBuffer::dynamic(display, PrimitiveType::LineStrip,
                           level.boundary_ix().as_slice()).unwrap()
    };

    LevelRenderer {
      level: level,
      display: display,
      render_stage: 0,
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
      cave_params: DrawParameters {
        line_width: Some(1.0),
        polygon_mode: PolygonMode::Fill,
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
    // First tick the simulation
    if !self.level.level_gen_finished {
      self.level.tick_level_gen();
    }
    let cave_bounds = boundary_verts(&self.level.boundary);

    // Then render
    // In the first 4 stages we draw the CA evolution and the boundary
    if self.level.gen_stage <= 3 {
      self.cave_bounds_vertb.write(&cave_bounds);
      self.cave_bounds_indxb.write(self.level.boundary_ix().as_slice());
      let cave_ca = cave_verts(&self.level.ca_grid);
      self.cave_ca_vertb.write(&cave_ca);
      frame.draw(&self.cave_ca_vertb, &NO_IXS, &self.ca_prog, &uniforms,
                 &self.ca_params).unwrap();
      frame.draw(&self.cave_bounds_vertb, &self.cave_bounds_indxb,
                 &self.bounds_prog, &uniforms, &self.bounds_params).unwrap();
    } else {
      // Next, we draw the whole cave as a polygon
      if self.render_stage == 0 {
        let polycave = self.cave_bounds_to_poly();
        self.cave_bounds_vertb.write(&polycave);

        self.render_stage += 1;
      }
      frame.draw(&self.cave_bounds_vertb, &NO_IXS_TRI,
                 &self.ca_prog, &uniforms, &self.cave_params).unwrap();
    }
  }

  pub fn stop_render(&mut self) -> () { self.level.level_gen_finished = true }

  fn cave_bounds_to_poly(&mut self) -> Vec<Vertex> {
    // We have to triangulate the boundary polygon. We use some helpful
    // code provided by Campbell Barton to do this. First, project
    // everything into unit space.
    let bounds_p: Vec<[f64; 2]> = self.level.boundary.iter()
      .map(|v| {
        let p = project_to_unitspace(v.0 as usize, v.1 as usize);
        [p.x as f64, p.y as f64]
      }).collect();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    polyfill_calc(&bounds_p, 0, &mut tris);
    // Need to convert the indexes (in tris) back into coordinates.
    let mut tri_list: Vec<Vertex> = tris.iter().flat_map(|x|
      x.iter().map(|xx| Vertex {
        pos: [bounds_p[*xx as usize][0] as f32,
          bounds_p[*xx as usize][1] as f32]
      })).collect();
    for _ in tri_list.len()..CA_BUFSIZ {
      // We're just putting them way off in the corner somewhere invisible
      tri_list.push(Vertex { pos: [-10.0, -10.0] });
    }
    tri_list
  }
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
