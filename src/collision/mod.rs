extern crate nalgebra as na;
extern crate ncollide as nc;

use na::Isometry2;
use nc::broad_phase::BroadPhasePairFilter;
use nc::ncollide_pipeline::world::{CollisionGroups, CollisionObjectHandle, CollisionObject2};
use nc::shape::{Compound2, Cuboid2, ShapeHandle2};
use util::{Meters, Point};

pub type CollW = nc::world::CollisionWorld2<Meters, CollidableDat>;
pub type CollisionRect = Cuboid2<Meters>;
pub type Shape2D = ShapeHandle2<Meters>;
pub type Compound2D = Compound2<Meters>;

pub fn new_collw() -> CollW {
  let mut retme = CollW::new(0.02);
  retme.register_broad_phase_pair_filter("Same entity filter", SameEntityFilter);
  retme
}

/// Trait that allows the implementer to be collidable with in the game world. This should be
/// pretty much everything on-screen that isn't UI.
pub trait Collidable {
  /// This object's location in world coordinates
  fn location(&self) -> Point;
  /// Defines how the object is collided with
  fn shape(&self) -> Shape2D;
  /// The collision group this object belongs to
  fn collision_group(&self) -> CollisionGroups;
  /// This collidable's type to be used in `CollidableDat`
  fn coltype(&self) -> CollidableType;
}

#[derive(new, Eq, PartialEq, Debug, Copy, Clone)]
pub struct CollidableDat {
  pub otype: CollidableType,
  pub id: usize,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum CollidableType {
  RoomWall,
  CompoundRoomWall,
}

pub trait GameObjRegistrar {
  fn register(&mut self, register_me: &Collidable, dat: CollidableDat) -> CollisionObjectHandle;
  fn register_with_group(&mut self, register_me: &Collidable, group: CollisionGroups,
                         dat: CollidableDat) -> CollisionObjectHandle;
}

impl GameObjRegistrar for CollW {
  fn register(&mut self, register_me: &Collidable, dat: CollidableDat) -> CollisionObjectHandle {
    self.register_with_group(register_me, register_me.collision_group(), dat)
  }
  fn register_with_group(&mut self, register_me: &Collidable, group: CollisionGroups,
                         dat: CollidableDat) -> CollisionObjectHandle {
    let q = nc::world::GeometricQueryType::Contacts(0.0, 0.0);
    self.add(Isometry2::new(register_me.location().coords, na::zero()), register_me.shape(),
             group, q, dat)
  }
}

struct SameEntityFilter;

impl BroadPhasePairFilter<Point, Isometry2<Meters>, CollidableDat> for SameEntityFilter {
  fn is_pair_valid(&self,
                   b1: &CollisionObject2<Meters, CollidableDat>,
                   b2: &CollisionObject2<Meters, CollidableDat>) -> bool {
    // Disable self-collision. Compound rooms for example are composed of multiple collidables
    // that might touch, with the same id.
    if b1.data().id == b2.data().id { return false; };
    true
  }
}