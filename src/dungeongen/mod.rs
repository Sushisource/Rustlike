#![allow(dead_code)]
extern crate rand;
extern crate noise;
extern crate nalgebra as na;

use std::fmt;
use std::f32;
use self::na::Vector2;

const VERTC: i32 = 5000;
const CAVE_RAD: f32 = 0.5;
const DIR_CHOICES: [Direction; 4] = [Direction::North, Direction::South,
  Direction::East, Direction::West];
const CA_W: usize = 200;
const CA_H: usize = 150;
const CA_BUFSIZ: usize = CA_H * CA_W;

type Point = Vector2<f32>;
type CavePoints = Vec<Point>;

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave: CavePoints
}

impl Level {
  pub fn gen_cave() -> Level {
    let mut ca_grid = [[false; CA_H]; CA_W];
    // First populate a random box in the middle of the grid
    let inner_box_w = CA_W / 4;
    let inner_box_h = CA_H / 4;
    let left_edge = (CA_W / 2) - (inner_box_w / 2);
    let top_edge = (CA_H / 2) - (inner_box_h / 2);
    for x in left_edge..(inner_box_w + left_edge) {
      for y in top_edge..(inner_box_h + top_edge) {
        ca_grid[x][y] = rand::random();
      }
    }
    let mut as_points: Vec<Point> = Vec::with_capacity(CA_W * CA_H);
    for x in 0..(CA_W - 2) {
      for y in 0..(CA_H - 2) {
        if ca_grid[x][y] {
          as_points.push(project_to_unitspace(x, y));
        }
      }
    }
    Level { cave: as_points }
  }

  pub fn cave_verts(&self) -> Vec<Vertex> {
    let mut verts = self.cave.iter().map(|&x| Vertex { position: [x.x, x.y] })
      .collect::<Vec<Vertex>>();
    // We have to pad the array so it's always the same size, so openGL doesn't
    // freak out when we update it with more or less verticies
    for _ in verts.len()..CA_BUFSIZ {
      // We're just putting them way off in the corner somewhere invisible
      verts.push(Vertex { position: [-10.0, -10.0] });
    }
    verts
  }
}

fn project_to_unitspace(x: usize, y: usize) -> Point {
  let xp = x as f32 / CA_W as f32;
  let yp = y as f32 / CA_H as f32;
  println!("x/y {:?}/{:?}", xp, yp);
  Point::new(xp, yp)
}

enum Direction {
  North,
  South,
  East,
  West
}

impl Direction {
  pub fn to_vec(&self) -> Point {
    let arr = match *self {
      Direction::North => [0.0, 1.0],
      Direction::South => [0.0, -1.0],
      Direction::East => [1.0, 0.0],
      Direction::West => [-1.0, 0.0]
    };
    Point::from(&arr)
  }
}

// TODO: Move vertex somewhere more graphics-specific
#[derive(Copy, Clone)]
pub struct Vertex {
  pub position: [f32; 2],
}

impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Vert: {:?}", self.position)
  }
}
implement_vertex!(Vertex, position);
