pub mod player;

extern crate ggez;

use ggez::graphics::{Point2, Vector2};

pub trait Agent {
  fn width(&self) -> u32;
  fn height(&self) -> u32;
  fn symbol(&self) -> &'static str;
  fn pos(&self) -> Point2;
  fn trans(&mut self, by: Vector2);
}
