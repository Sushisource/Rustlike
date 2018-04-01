extern crate ggez;
extern crate ncollide;

use ggez::{Context, GameResult};
use ggez::graphics::{Point2, DrawParam, rectangle, Rect, DrawMode};
use self::ncollide::bounding_volume::AABB;
use dungeongen::level_renderer::LevelPoint;
use util::Meters;

pub trait ContextHelp {
  fn screen_x(&self) -> f32;
  fn screen_y(&self) -> f32;
  fn screen_middle(&self) -> Point2;
  fn sscale(&self) -> DrawParam;
  fn uspace_to_sspace(&self, p: LevelPoint) -> Point2;
  fn sspace_to_uspace(&self, p: Point2) -> Point2;
  fn center_rect(&mut self, center: Point2, w: Meters, h: Meters) -> GameResult<()>;
  fn draw_bb(&mut self, bb: &AABB<Point2>) -> GameResult<()>;
}

impl ContextHelp for Context {
  fn screen_x(&self) -> f32 {
    return self.conf.window_mode.width as f32;
  }

  fn screen_y(&self) -> f32 {
    return self.conf.window_mode.height as f32;
  }

  fn screen_middle(&self) -> Point2 {
    Point2::new(self.screen_x() / 2.0, self.screen_y() / 2.0)
  }

  fn sscale(&self) -> DrawParam {
    DrawParam {
      scale: Point2::new(self.screen_x(), self.screen_y()),
      dest: self.screen_middle(),
      ..Default::default()
    }
  }

  fn uspace_to_sspace(&self, p: LevelPoint) -> Point2 {
    let sx = self.screen_x();
    let sy = self.screen_y();
    Point2::new(p.x * sx, p.y * sy)
  }

  fn sspace_to_uspace(&self, p: Point2) -> Point2 {
    Point2::new(p.x / self.screen_x(), p.y / self.screen_y())
  }

  fn center_rect(&mut self, c: Point2, w: Meters, h: Meters) -> GameResult<()> {
    rectangle(self, DrawMode::Fill,
              Rect {
                x: c.coords.x - w / 2.0,
                y: c.coords.y - h / 2.0,
                w,
                h,
              })
  }

  fn draw_bb(&mut self, bb: &AABB<Point2>) -> GameResult<()> {
    let bh = bb.maxs().y - bb.mins().y;
    let bw = bb.maxs().x - bb.mins().x;
    self.center_rect(bb.center(), bw, bh)
  }
}