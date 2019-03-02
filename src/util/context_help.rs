use crate::nc::bounding_volume::AABB;
use crate::util::{Meters, Point};
use crate::util::Vec2;
use ggez::{
  graphics::Mesh,
  graphics::draw,
  Context,
  GameResult,
  graphics::{DrawParam, Rect},
  graphics::Color,
  graphics::DrawMode,
};

pub trait ContextHelp {
  fn screen_x(&self) -> f32;
  fn screen_y(&self) -> f32;
  fn screen_middle(&self) -> Point;
  fn sscale(&self) -> DrawParam;
  fn uspace_to_sspace(&self, p: Point) -> Point;
  fn sspace_to_uspace(&self, p: Point) -> Point;
  fn center_rect(&mut self, center: Point, w: Meters, h: Meters, color: Color) -> GameResult<()>;
  fn draw_bb(&mut self, bb: &AABB<Meters>) -> GameResult<()>;
}

impl ContextHelp for Context {
  fn screen_x(&self) -> f32 {
    self.conf.window_mode.width as f32
  }

  fn screen_y(&self) -> f32 {
    self.conf.window_mode.height as f32
  }

  fn screen_middle(&self) -> Point {
    Point::new(self.screen_x() / 2.0, self.screen_y() / 2.0)
  }

  fn sscale(&self) -> DrawParam {
    DrawParam {
      scale: Vec2::new(self.screen_x(), self.screen_y()).into(),
      dest: self.screen_middle().into(),
      ..Default::default()
    }
  }

  fn uspace_to_sspace(&self, p: Point) -> Point {
    let sx = self.screen_x();
    let sy = self.screen_y();
    Point::new(p.x * sx, p.y * sy)
  }

  fn sspace_to_uspace(&self, p: Point) -> Point {
    Point::new(p.x / self.screen_x(), p.y / self.screen_y())
  }

  fn center_rect(&mut self, c: Point, w: Meters, h: Meters, color: Color) -> GameResult<()> {
    let r = Rect { x: c.coords.x - w / 2.0, y: c.coords.y - h / 2.0, w, h };
    let r = Mesh::new_rectangle(self, DrawMode::fill(), r, color)?;
    draw(self, &r, DrawParam::new())
  }

  fn draw_bb(&mut self, bb: &AABB<Meters>) -> GameResult<()> {
    let bh = bb.maxs().y - bb.mins().y;
    let bw = bb.maxs().x - bb.mins().x;
    self.center_rect(bb.center(), bw, bh, Color::new(0.0, 1.0, 0.0, 0.5))
  }
}
