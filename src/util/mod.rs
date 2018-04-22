extern crate ggez;
extern crate nalgebra as na;

use self::ggez::Context;
use self::ggez::graphics::{Font, Text};
use std::collections::HashMap;
use super::agents::Agent;

pub mod context_help;

pub type Meters = f32;
pub type Point = na::Point2<f32>;

pub struct Assets {
  /// This map maps world sizes in meters -> font where the size as rendered
  /// without scaling is equal to that world size.
  font_map: HashMap<u32, Font>,
  // This maps strings to their text objects so we don't need to
  // build text objects over and over
  text_map: HashMap<&'static str, Text>,
}

impl Assets {
  pub fn new(ctx: &mut Context) -> Assets {
    let mut m = HashMap::new();
    m.insert(1, Font::new(ctx, "/Hack-Bold.ttf", 14).unwrap());
    Assets {
      font_map: m,
      text_map: HashMap::new(),
    }
  }

  pub fn agent_txt<T: Agent>(&mut self, agent: &T, ctx: &mut Context) -> &Text {
    // TODO: Could be crashy
    self.text_map.entry(agent.symbol()).or_insert(
      Text::new(
        ctx,
        agent.symbol(),
        self.font_map.get(&agent.width()).unwrap(),
      ).unwrap(),
    )
  }

  pub fn txt(&mut self, content: &str, ctx: &mut Context) -> Text {
    Text::new(ctx, content, self.font_map.get(&1).unwrap()).unwrap()
  }
}
