pub mod direction;
pub mod level;

mod blobstacle;
mod ca_simulator;
mod compound_room;
mod rooms;

use crate::{
  dungeongen::{ca_simulator::CASim, level::Level},
  world,
};
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::{
  prelude::*,
  render::texture::{Extent3d, TextureDimension, TextureFormat},
};

pub struct BoundsDrawer;

/// The system for dungeon generation.
pub fn dungeongen_init(
  commands: &mut Commands,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut stdmat: ResMut<Assets<StandardMaterial>>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  // Init the CA draw target
  let mathandle = materials.set("ca_texmat", ColorMaterial::default());
  commands.spawn(SpriteBundle {
    sprite: Sprite::new(Vec2::new(800., 800.)),
    transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
    material: mathandle,
    ..Default::default()
  });

  // Draw bounding box / background
  commands.spawn(SpriteBundle {
    sprite: Sprite::new(Vec2::new(800., 800.)),
    transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
    material: materials.add(Color::rgba(0., 0., 0., 1.).into()),
    ..Default::default()
  });

  let mut mesh = Mesh::new(PrimitiveTopology::LineList);
  // Bevy crashes w/o any indicies
  mesh.set_indices(Some(Indices::U32(vec![0])));
  let mesh_h = meshes.set("ca_boundary", mesh);
  commands.spawn((
    BoundsDrawer,
    PbrBundle {
      mesh: mesh_h,
      transform: Transform::from_translation(Vec3::new(0., 0., 2.)),
      material: stdmat.add(StandardMaterial::from(Color::rgb(1., 0., 0.))),
      ..Default::default()
    },
  ));
}

/// The system for dungeon generation.
pub fn dungeongen(
  // commands: &mut Commands,
  mut state: ResMut<world::World>,
) {
  if !state.level.level_gen_finished {
    // Speed up generation. Might be nice to see if there's a more ecs-friendly way to deal with it
    for _ in 1..10 {
      state.level.tick_level_gen();
    }
  }
}

pub fn draw_evolution(
  mut textures: ResMut<Assets<Texture>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut state: ResMut<world::World>,
) {
  if state.level.cave_sim.gen_stage == 0 {
    let ca_img_a = state.level.cave_sim.cave_ca_img();
    let texture = Texture::new_fill(
      // TODO: Get dimensions from constant or whatever
      Extent3d::new(200, 200, 1),
      TextureDimension::D2,
      &ca_img_a,
      TextureFormat::Bgra8Unorm,
    );
    let texhandle = textures.set("ca_texture", texture);
    materials.set("ca_texmat", ColorMaterial::texture(texhandle));
  }
}

pub fn draw_ca_boundary(mut state: ResMut<world::World>, mut meshes: ResMut<Assets<Mesh>>) {
  if state.level.cave_sim.gen_stage == 1 {
    let cave_bounds = state.level.cave_sim.uspace_gboundary();
    if !cave_bounds.is_empty() {
      let indices = Indices::U32((0u32..cave_bounds.len() as u32).into_iter().collect());
      let positions: Vec<_> = cave_bounds.into_iter().map(|p| [p.x, p.y]).collect();
      let m = meshes.get_mut("ca_boundary").unwrap();
      log::info!("Pump up the JAM");
      m.set_indices(Some(indices));
      m.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    }
  }
}
