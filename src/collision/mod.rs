extern crate nalgebra as na;
extern crate ncollide as nc;

use self::na::{Isometry2};
use self::nc::ncollide_pipeline::world::CollisionGroups;
use self::nc::shape::{Cuboid2, ShapeHandle2, Compound2};
use util::{Meters, Point};

// TODO: Second param is wrong, represents defined-by-me data. Need
// to have some kind of ID system for all entities
pub type CollW = nc::world::CollisionWorld2<Meters, f32>;
pub type CollisionRect = Cuboid2<Meters>;
pub type Shape2D = ShapeHandle2<Meters>;
pub type Compound2D = Compound2<Meters>;

/// Trait that allows the implementer to be collidable with in the game world. This should be
/// pretty much everything on-screen that isn't UI.
pub trait Collidable {
  /// This object's location in world coordinates
  fn location(&self) -> Point;
  /// Defines how the object is collided with
  fn shape(&self) -> Shape2D;
  /// The collision group this object belongs to
  fn collision_group(&self) -> CollisionGroups;
}

pub trait GameObjRegistrar {
  fn register(&mut self, register_me: &Collidable) -> ();
}

impl GameObjRegistrar for CollW {
  fn register(&mut self, register_me: &Collidable) -> () {
    let q = nc::world::GeometricQueryType::Contacts(0.0, 0.0);
    self.add(Isometry2::new(register_me.location().coords, na::zero()), register_me.shape(),
             register_me.collision_group(), q, 0.0);
  }
}