extern crate ggez;

use ggez::Context;
use self::ggez::graphics::{Point2, DrawParam};
use dungeongen::level_renderer::LevelPoint;

pub trait ContextHelp {
  fn screen_x(&self) -> f32;
  fn screen_y(&self) -> f32;
  fn screen_middle(&self) -> Point2;
  fn sscale(&self) -> DrawParam;
  fn uspace_to_sspace(&self, p: LevelPoint) -> Point2;
  fn sspace_to_uspace(&self, p: Point2) -> Point2;
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
}