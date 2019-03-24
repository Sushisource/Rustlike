pub mod mouse_mover;
pub mod player;

use crate::util::Point;
use crate::util::Vec2;

pub trait Agent {
  fn width(&self) -> u32;
  fn height(&self) -> u32;
  fn symbol(&self) -> &'static str;
  fn pos(&self) -> Point;
  fn trans(&mut self, by: Vec2);
}
