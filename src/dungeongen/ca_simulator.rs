extern crate rand;
extern crate ggez;

use self::ggez::{Context, GameResult};
use self::ggez::graphics;
use self::ggez::graphics::{DrawParam, Drawable, FilterMode, Image};
use self::ggez::graphics::Point as GPoint;

use super::direction::Direction;
use super::Point;
use super::level_renderer::DrawablePt;

const CA_W: usize = 266;
const CA_H: usize = 150;

type CellGrid = [[bool; CA_H]; CA_W];

// TODO: Make drawable?
pub struct CASim {
  pub ca_grid: CellGrid,
  pub ca_boundary: Vec<(i32, i32)>,
  scale: f32,
  gen_stage: u8,
  bounds_last_dir: Direction
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

// TODO: Make this drawable?
impl CASim {
  pub fn new(scale: f32) -> CASim {
    let ca_grid = gen_cave();
    CASim {
      ca_grid,
      ca_boundary: Vec::new(),
      scale,
      gen_stage: 0,
      bounds_last_dir: Direction::SouthEast
    }
  }

  pub fn generate(&mut self) {
    while !self.tick() {}
  }

  pub fn tick(&mut self) -> bool {
    let stage_complete = match self.gen_stage {
      0 => self.tick_ca_sim(),
      1 => self.tick_cave_boundary(),
      2 => self.smooth_cave_boundary(),
      3 => {
        // Make sure boundary is fully conected
        // TODO: Move this part to renderer?
        let back_to_first = self.ca_boundary[0].clone();
        self.ca_boundary.push(back_to_first);
        true
      }
      _ => false,
    };
    if stage_complete {
      self.gen_stage += 1
    }
    self.gen_stage > 3
  }

  /// Converts cellular automata space to unit space
  pub fn uspace_boundary(&self) -> Vec<Point> {
    self.ca_boundary.iter().map(|&(x, y)| {
      let xp = (x as f32) / (CA_W as f32) * self.scale;
      let yp = (y as f32) / (CA_H as f32) * self.scale;
      Point::new(xp, yp)
    }).collect()
  }

  fn smooth_cave_boundary(&mut self) -> bool {
    let mut a = 0;
    self.ca_boundary.retain(|_| {
      a += 1;
      a % 2 == 0
    });
    true
  }

  fn tick_cave_boundary(&mut self) -> bool {
    // Inspect grid, starting top left and work around clockwise building poly
    let mut cur_cell = (0, 0);
    // First I move in from the corner until I hit a cell, if this is the first
    // tick.
    if self.ca_boundary.is_empty() {
      'out: for x in 0..(CA_W - 1) {
        for y in 0..(CA_H - 1) {
          if self.ca_grid[x][y] {
            cur_cell = (x as i32, y as i32);
            break 'out;
          }
        }
      }
      self.ca_boundary.push(cur_cell);
    }
    cur_cell = *self.ca_boundary.last().unwrap();
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
      let cur_pt = dir.dir_from_tup(cur_cell);
      // Bounds check, followed by cell present check
      let in_width = cur_pt.0 >= 0 && cur_pt.0 <= CA_W as i32;
      let in_height = cur_pt.1 >= 0 && cur_pt.1 <= CA_H as i32;
      let not_marked = !self.ca_boundary.contains(&cur_pt);
      if !not_marked {
        marked_ct += 1;
      }
      if in_width && in_height &&
        self.ca_grid[cur_pt.0 as usize][cur_pt.1 as usize] &&
        not_marked {
        cur_cell = cur_pt;
        self.ca_boundary.push(cur_cell);
        self.bounds_last_dir = dir.clone();
        break;
      }
    }
    if marked_ct >= 2 {
      println!("Done drawing cave boundary");
      true
    } else {
      false
    }
  }


  fn tick_ca_sim(&mut self) -> bool {
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
            // finished.
            if x == 0 || x == CA_W - 1 || y == 0 || y == CA_H - 2 {
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
          if nc == 1 || nc == 0 {
            self.ca_grid[x][y] = false;
          }
        }
      }
      println!("Done simulating CA for cave");
    }
    growth_done
  }

  fn neighbor_count(&self, x: usize, y: usize) -> u8 {
    let mut count = 0;
    if x >= 1 {
      if y >= 1 && self.ca_grid[x - 1][y - 1] {
        count += 1
      };
      if self.ca_grid[x - 1][y] {
        count += 1
      };
      if self.ca_grid[x - 1][y + 1] {
        count += 1
      };
    }
    if y >= 1 && self.ca_grid[x][y - 1] {
      count += 1
    };
    if self.ca_grid[x][y + 1] {
      count += 1
    };
    if y >= 1 && self.ca_grid[x + 1][y - 1] {
      count += 1
    };
    if self.ca_grid[x + 1][y] {
      count += 1
    };
    if self.ca_grid[x + 1][y + 1] {
      count += 1
    };
    count
  }

  // GRAPHICS =================================================================
  // Not really sure it's good practice to put this here, but I can't put it
  // in Drawable impl.
  pub fn draw_evolution(&self, ctx: &mut Context, param: DrawParam)
                        -> GameResult<()> {
    let ca_img_a = self.cave_ca_img(&self.ca_grid);
    let screen_scale = DrawablePt(Point::new(param.scale.x, param.scale.y));
    let scalept = DrawablePt(
      Point::new(1.0 / CA_W as f32, 1.0 / CA_H as f32)) * screen_scale;
    let mut img = Image::from_rgba8(ctx, CA_W as u16, CA_H as u16, &ca_img_a)?;
    let mut scaled_params = param.clone();
    scaled_params.scale = scalept.into();
    // Don't make my pixels all blurry
    img.set_filter(FilterMode::Nearest);
    img.draw_ex(ctx, scaled_params)?;

    // Boundary drawing
    let cave_bounds: Vec<GPoint> = self.uspace_boundary().iter()
                                       .map(|&p| {
                                         let scaled = (DrawablePt(p) * screen_scale).snap();
                                         scaled.into()
                                       })
                                       .collect();
    if !cave_bounds.is_empty() {
      graphics::set_line_width(ctx, 4.0);
      graphics::line(ctx, cave_bounds.as_slice())?;
    }
    Ok(())
  }

  /// Converts the cave CA sim to a 1d array of RGBA 8 bit values
  fn cave_ca_img(&self, cell_grid: &CellGrid) -> [u8; CA_W * CA_H * 4] {
    let mut img = [0; CA_W * CA_H * 4];
    for x in 0..(CA_W - 1) {
      for y in 0..(CA_H - 1) {
        if cell_grid[x][y] {
          let i = (CA_W * y + x) * 4;
          img[i] = 0xAF;
          img[i + 1] = 0xAF;
          img[i + 2] = 0xAF;
          img[i + 3] = 0xFF;
        }
      }
    }
    img
  }
}

//impl Drawable for CASim {
//  fn draw_ex(&self, ctx: &mut Context, param: DrawParam) -> GameResult<()> {
//    //    let boundary = self.upts_to_sspace(&obstacle.uspace_boundary());
//    //    graphics::set_color(ctx, Color::new(0.5, 0.5, 0.8, 1.0))?;
//    //    graphics::polygon(ctx, DrawMode::Fill, boundary.as_slice())?;
//    Ok(())
//  }
//}
