pub mod player;

extern crate ggez;

use super::util::Point;

pub trait Agent {
  fn width(&self) -> u32;
  fn height(&self) -> u32;
  fn symbol(&self) -> &'static str;
  fn pos(&self) -> Point;
}
