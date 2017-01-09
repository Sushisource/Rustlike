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
type CellGrid = [[bool; CA_H]; CA_W];

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave: CavePoints,
  pub ca_grid: CellGrid
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
    Level::new(ca_grid)
  }

  pub fn new(ca_grid: CellGrid) -> Level {
    let as_points = Level::cave_from_grid(ca_grid);
    Level { cave: as_points, ca_grid: ca_grid }
  }
  fn update_cave(&mut self) -> () {
    let as_points = Level::cave_from_grid(self.ca_grid);
    self.cave = as_points;
  }
  fn cave_from_grid(ca_grid: CellGrid) -> CavePoints {
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

  pub fn tick_cavesim(&mut self) -> () {
    let mut ca_grid_next = [[false; CA_H]; CA_W];
    {
      let neighbor_count = |x: usize, y: usize| -> i32 {
        let mut count = 0;
        if x >= 1 {
          if y >= 1 && self.ca_grid[x - 1][y - 1] { count += 1 };
          if self.ca_grid[x - 1][y] { count += 1 };
          if self.ca_grid[x - 1][y + 1] { count += 1 };
        }
        if y >= 1 && self.ca_grid[x][y - 1] { count += 1 };
        if self.ca_grid[x][y + 1] { count += 1 };
        if y >= 1 && self.ca_grid[x + 1][y - 1] { count += 1 };
        if self.ca_grid[x + 1][y] { count += 1 };
        if self.ca_grid[x + 1][y + 1] { count += 1 };
        count
      };

      for x in 0..(CA_W - 1) {
        for y in 0..(CA_H - 1) {
          let nc = neighbor_count(x, y);
          if self.ca_grid[x][y] {
            // Check for survival
            if nc >= 4 {
              // Cell survives
              ca_grid_next[x][y] = true;
            }
            // Cell dead
          } else if nc == 3 || nc >= 7 {
            // Cell born
            ca_grid_next[x][y] = true;
          }
        }
      }
    }
    self.ca_grid = ca_grid_next;
    self.update_cave();
  }
}

fn project_to_unitspace(x: usize, y: usize) -> Point {
  let xp = (x as f32) / (CA_W as f32);
  let yp = (y as f32) / (CA_H as f32);
  //  println!("x/y {:?}/{:?} -> {:?}/{:?}", x, y, xp, yp);
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
