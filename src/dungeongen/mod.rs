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
use bevy::{
  prelude::*,
  render::{
    mesh::Indices,
    pipeline::PrimitiveTopology,
    texture::{Extent3d, TextureDimension, TextureFormat},
  },
};
use lyon::algorithms::walk::Pattern;
use lyon::{
  math::{point, Point},
  path::{builder::*, Path},
  tessellation::*,
};

pub struct BoundsDrawer;

/// The system for dungeon generation.
pub fn dungeongen_init(
  commands: &mut Commands,
  mut materials: ResMut<Assets<ColorMaterial>>,
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

  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.set_indices(Some(Indices::U32(vec![0])));
  mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0., 0.]]);
  // TODO: Can be removed w/ custom shader that doesn't require them.
  mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0.]]);
  mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]]);
  let mesh_h = meshes.set("ca_boundary", mesh);
  let mut transform = Transform::from_translation(Vec3::new(0., 0., 2.));
  // TODO: properly scale? oh yay more fun
  transform.scale = [1., 1., 1.].into();
  commands.spawn(SpriteBundle {
    mesh: mesh_h,
    transform,
    material: materials.add(Color::rgba(1., 1., 0., 1.).into()),
    sprite: Sprite { size: Vec2::new(800.0, 800.0), ..Default::default() },
    ..Default::default()
  });
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
      TextureFormat::Rgba8Unorm,
    );
    let texhandle = textures.set("ca_texture", texture);
    materials.set("ca_texmat", ColorMaterial::texture(texhandle));
  }
}

pub fn draw_ca_boundary(mut state: ResMut<world::World>, mut meshes: ResMut<Assets<Mesh>>) {
  if state.level.cave_sim.gen_stage == 1 {
    let cave_bounds = state.level.cave_sim.uspace_gboundary();
    if !cave_bounds.is_empty() {
      let mut builder = Path::builder();
      builder.move_to(point(0.0, 0.0));
      for boundary_pt in &cave_bounds {
        builder.line_to(point(boundary_pt.x, boundary_pt.y));
      }
      builder.close();
      let path = builder.build();
      // Will contain the result of the tessellation.
      let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
      let mut tessellator = FillTessellator::new();
      {
        // Compute the tessellation.
        tessellator
          .tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: FillAttributes| pos.to_array()),
          )
          .unwrap();
      }

      let mut m = meshes.get_mut("ca_boundary").unwrap();
      m.set_indices(Some(Indices::U32(geometry.indices)));
      let mut normals: Vec<[f32; 3]> = Vec::new();
      let mut uvs: Vec<[f32; 2]> = Vec::new();
      for _ in 0..geometry.vertices.len() {
        normals.push([0.0, 0.0, 0.0]);
        uvs.push([0.0, 0.0]);
      }
      m.set_attribute(Mesh::ATTRIBUTE_POSITION, geometry.vertices);
      // TODO: Can be removed w/ custom shader that doesn't require them.
      m.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
      m.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
  }
}
