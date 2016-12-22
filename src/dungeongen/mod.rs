#![allow(dead_code)]
extern crate rand;
extern crate noise;
extern crate nalgebra as na;

use std::f32::consts::PI;
use std::fmt;
use std::f32;
use self::rand::Rng;
use self::noise::{Brownian2, Seed};
use self::na::Vector2;

const VERTC: i32 = 5000;
const CAVE_RAD: f32 = 0.5;
const DIR_CHOICES: [Direction; 4] = [Direction::North, Direction::South,
  Direction::East, Direction::West];

type Point = Vector2<f32>;
type CavePoints = Vec<Point>;

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave: CavePoints
}

impl Level {
  pub fn make_circle_cave() -> Level {
    let mut verts: CavePoints = Vec::new();
    for rp in 0..VERTC {
      let next_pt = {
        let prev_point = &verts.last();
        get_cave_pt(rp as f32, prev_point)
      };
      verts.push(next_pt);
    }
    // Close the gap
    let v1 = verts[0];
    verts.push(v1);
    Level { cave: verts }
  }

  pub fn make_walk_cave() -> Level {
    let mut rng = rand::thread_rng();
    let seed: Seed = Seed::new(rng.next_u32());
    let noise = Brownian2::new(noise::perlin2, 4).wavelength(100.0);

    let mut last_point = Point::new(0.1, 0.1);
    let mut verts: CavePoints = Vec::new();
    //    let dirref = &DIR_CHOICES;

    for _ in 0..VERTC {
      verts.push(last_point);
      let val = noise.apply(&seed, last_point.as_ref()) + 0.05;
      let dir: [f32; 2] = [rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)];
      println!("Val: {:?}  Dir: {:?}", val, dir);
      last_point = last_point + (Point::from(&dir) * val);
    }
    Level { cave: verts }
  }

  pub fn cave_verts(&self) -> Vec<Vertex> {
    self.cave.iter().map(|&x| Vertex { position: [x.x, x.y] })
      .collect::<Vec<Vertex>>()
  }
}

fn get_cave_pt(index: f32, prev_point_o: &Option<&Point>) -> Point {
  let mut rng = rand::thread_rng();
  let r_r = rng.gen_range(-0.1, 0.1);
  let prev_drift = if let &Some(prev_point) = prev_point_o {
    let prev_angle = 2.0 * PI * ((index - 1.0) / VERTC as f32);
    let prev_pos = [CAVE_RAD * prev_angle.cos(), CAVE_RAD * prev_angle.sin()];
    (prev_point.x - prev_pos[0], prev_point.y - prev_pos[1])
  } else {
    (0.0, 0.0)
  };
  let angle = 2.0 * PI * (index / VERTC as f32);
  let pos = [(CAVE_RAD + r_r) * angle.cos(), (CAVE_RAD + r_r) * angle.sin()];
  Point::new(pos[0] + prev_drift.0, pos[1] + prev_drift.1)
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
