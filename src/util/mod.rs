extern crate nalgebra as na;

use super::agents::Agent;
use bevy::prelude::Text;
use bevy::text::Font;
use std::collections::HashMap;

pub mod context_help;
pub mod geom;

pub type Meters = f32;
pub type Point = na::Point2<f32>;
pub type Vec2 = na::Vector2<f32>;

pub struct Assets {
  /// This map maps world sizes in meters -> font where the size as rendered
  /// without scaling is equal to that world size.
  font_map: HashMap<u32, Font>,
  // This maps strings to their text objects so we don't need to
  // build text objects over and over
  text_map: HashMap<&'static str, Text>,
}

// impl Assets {
//   pub fn new() -> Assets {
//     let mut m = HashMap::new();
//     m.insert(1, Font::new(ctx, "/Hack-Bold.ttf").unwrap());
//     Assets { font_map: m, text_map: HashMap::new() }
//   }
//
//   pub fn agent_txt<T: Agent>(&mut self, agent: &T) -> &Text {
//     // TODO: Could be crashy
//     let font = self.font_map[&agent.width()];
//     let text_frag = TextFragment {
//       color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
//       font: Some(font),
//       scale: None,
//       text: agent.symbol().to_string(),
//     };
//     self.text_map.entry(agent.symbol()).or_insert_with(|| Text::new(text_frag))
//   }
//
//   pub fn txt(&mut self, content: &str) -> Text {
//     let font = self.font_map[&1];
//     let text_frag = TextFragment {
//       color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
//       font: Some(font),
//       scale: None,
//       text: content.to_string(),
//     };
//     Text::new(text_frag)
//   }
// }
