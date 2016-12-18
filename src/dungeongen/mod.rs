extern crate rand;

use self::rand::Rng;
use std::f32;
use std::f32::consts::PI;
use std::fmt;

const VERTC: i32 = 100;
const CAVE_RAD: f32 = 0.5;

/// A level consists of one huge arbitrarily-shaped but enclosed curve, on top
/// of which we will layer features. This bottom layer represents the shape of
/// the cavern.
pub struct Level {
  pub cave: Vec<Vertex>,
}

impl Level {
  pub fn make_circle_cave(&self) -> Level {
    let mut verts: Vec<Vertex> = Vec::new();
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
}

fn get_cave_pt(index: f32, prev_point_o: &Option<&Vertex>) -> Vertex {
  let mut rng = rand::thread_rng();
  let r_r = rng.gen_range(-0.1, 0.1);
  let prev_drift = if let &Some(prev_point) = prev_point_o {
    let prev_angle = 2.0 * PI * ((index - 1.0) / VERTC as f32);
    let prev_pos = [CAVE_RAD * prev_angle.cos(), CAVE_RAD * prev_angle.sin()];
    (prev_point.position[0] - prev_pos[0], prev_point.position[1] - prev_pos[1])
  } else {
    (0.0, 0.0)
  };
  let angle = 2.0 * PI * (index / VERTC as f32);
  let pos = [(CAVE_RAD + r_r) * angle.cos(), (CAVE_RAD + r_r) * angle.sin()];
  Vertex {
    position: [pos[0] + prev_drift.0, pos[1] + prev_drift.1]
  }
}

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
