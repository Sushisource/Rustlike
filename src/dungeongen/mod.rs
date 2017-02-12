#![allow(dead_code)]
extern crate rand;
extern crate noise;
extern crate nalgebra as na;

pub mod level_renderer;
pub mod direction;

use std::f32;
use self::na::Vector2;
use self::level_renderer::Vertex;
use self::direction::Direction;

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
  pub ca_grid: CellGrid,
  pub boundary: Vec<(i32, i32)>,
  pub level_gen_finished: bool,
  gen_stage: u8,
  bounds_last_dir: Direction
}

impl Level {
  pub fn new() -> Level {
    let ca_grid = Level::gen_cave();
    let as_points = Level::cave_from_grid(ca_grid);
    Level {
      cave: as_points,
      ca_grid: ca_grid,
      boundary: Vec::new(),
      level_gen_finished: false,
      gen_stage: 0,
      bounds_last_dir: Direction::SouthEast
    }
  }

  fn gen_cave() -> [[bool; CA_H]; CA_W] {
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
    ca_grid
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

  // TODO: Maybe move into renderer?
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

  pub fn boundary_verts(&self) -> Vec<Vertex> {
    let mut verts = self.boundary.iter().map(|&(x, y)| {
      let as_pt = project_to_unitspace(x as usize, y as usize);
      Vertex { position: [as_pt.x, as_pt.y] }
    }).collect::<Vec<Vertex>>();
    for _ in verts.len()..CA_BUFSIZ {
      // We're just putting them way off in the corner somewhere invisible
      verts.push(Vertex { position: [-10.0, -10.0] });
    }
    verts
  }

  pub fn boundary_ix(&self) -> Vec<u16> {
    let x = self.boundary.len() as u16;
    let second_half: Vec<u16> = (0..x).collect();
    let mut first = vec![0; CA_BUFSIZ - second_half.len()];
    first.extend(second_half);
    first
  }

  pub fn tick_level_gen(&mut self) -> () {
    let stage_complete = match self.gen_stage {
      0 => self.tick_cavesim(),
      1 => self.tick_cave_boundary(),
      2 => self.smooth_cave_boundary(),
      3 => {
        // Make sure boundary is fully conected
        let back_to_first = self.boundary[0].clone();
        self.boundary.push(back_to_first);
        true
      }
      _ => false,
    };
    if stage_complete { self.gen_stage += 1 }
  }

  fn smooth_cave_boundary(&mut self) -> bool {
    let mut a = 0;
    self.boundary.retain(|_| {
      a += 1;
      a % 2 == 0
    });
    true
  }

  fn tick_cave_boundary(&mut self) -> bool {
    // Inspect grid, starting top left and work around clockwise building poly
    let mut cur_pixel = (0, 0);
    // First I move in from the corner until I hit a cell, if this is the first
    // tick.
    if self.boundary.is_empty() {
      'out: for x in 0..(CA_W - 1) {
        for y in 0..(CA_H - 1) {
          if self.ca_grid[x][y] {
            cur_pixel = (x as i32, y as i32);
            break 'out;
          }
        }
      }
      self.boundary.push(cur_pixel);
    }
    cur_pixel = *self.boundary.last().unwrap();
    // Then we will use a radial sweep algorithm to trace the boundary of the
    // cells. Starting from the current point, we check it's neighbors in a
    // clockwise fashion until we find another occupied cell.

    // Start the sweep one tick clockwise from the direction we just came from
    let in_dir = self.bounds_last_dir.opposite();
    let dirs = {
      let first = Direction::iterator().skip_while(|x| **x != in_dir).skip(1);
      let rest = Direction::iterator().take_while(|x| **x != in_dir);
      first.chain(rest)
    };
    let mut marked_ct = 0;
    for dir in dirs {
      let cur_pt = dir.dir_from_tup(cur_pixel);
      // Bounds check, followed by cell present check
      let in_width = cur_pt.0 >= 0 && cur_pt.0 <= CA_W as i32;
      let in_height = cur_pt.1 >= 0 && cur_pt.1 <= CA_H as i32;
      let not_marked = !self.boundary.contains(&cur_pt);
      if !not_marked {
        marked_ct += 1;
      }
      if in_width && in_height
        && self.ca_grid[cur_pt.0 as usize][cur_pt.1 as usize] && not_marked {
        cur_pixel = cur_pt;
        self.boundary.push(cur_pixel);
        self.bounds_last_dir = dir.clone();
        break;
      }
    }
    if marked_ct >= 2 { true } else { false }
  }

  fn tick_cavesim(&mut self) -> bool {
    let mut growth_done = false;
    let mut ca_grid_next = [[false; CA_H]; CA_W];
    {
      for x in 0..(CA_W - 1) {
        for y in 0..(CA_H - 1) {
          let nc = self.neighbor_count(x, y);
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
            // Check if it was born at the boundary, which means the sim is
            // finished. TODO: This is wonky, doesn't catch top sometimes?
            if x == 0 || x == CA_W - 1 || y == 0 || y == CA_H - 1 {
              growth_done = true;
            }
          }
        }
      }
    }
    self.ca_grid = ca_grid_next;
    if growth_done {
      // Trim all the "danglers" - these prevent boundary from forming
      for x in 0..(CA_W - 1) {
        for y in 0..(CA_H - 1) {
          let nc = self.neighbor_count(x, y);
          if nc == 1 {
            self.ca_grid[x][y] = false;
          }
        }
      }
      println!("Done simulating CA for cave");
    }
    self.update_cave();
    growth_done
  }

  fn neighbor_count(&self, x: usize, y: usize) -> i32 {
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
  }
}


fn project_to_unitspace(x: usize, y: usize) -> Point {
  let xp = (x as f32) / (CA_W as f32) - 0.5;
  let yp = (y as f32) / (CA_H as f32) - 0.5;
  //  println!("x/y {:?}/{:?} -> {:?}/{:?}", x, y, xp, yp);
  Point::new(xp * 2.0, yp * 2.0)
}
