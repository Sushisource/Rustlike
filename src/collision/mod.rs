extern crate nalgebra as na;
extern crate ncollide2d as nc;

use crate::util::Meters;
use na::Isometry2;
use nc::bounding_volume::AABB;
use nc::broad_phase::BroadPhasePairFilter;
use nc::shape::{Compound, Cuboid, ShapeHandle};
use nc::world::{CollisionGroups, CollisionObject, CollisionObjectHandle};

pub type CollW = nc::world::CollisionWorld<Meters, CollidableDat>;
pub type CollisionRect = Cuboid<Meters>;
pub type Shape2D = ShapeHandle<Meters>;
pub type Compound2D = Compound<Meters>;

pub fn new_collw() -> CollW {
  let mut retme = CollW::new(0.02);
  retme.register_broad_phase_pair_filter("Same entity filter", SameEntityFilter);
  retme
}

/// Trait that allows the implementer to be collidable with in the game world. This should be
/// pretty much everything on-screen that isn't UI.
pub trait Collidable {
  /// This object's location in world coordinates
  fn location(&self) -> Isometry2<Meters>;
  /// Defines how the object is collided with
  fn shape(&self) -> Shape2D;
  /// The collision group this object belongs to
  fn collision_group(&self) -> CollisionGroups;
  /// This collidable's type to be used in `CollidableDat`
  fn coltype(&self) -> CollidableType;
}

impl Collidable for AABB<Meters> {
  fn location(&self) -> Isometry2<Meters> {
    Isometry2::new(self.center().coords, na::zero())
  }
  fn shape(&self) -> Shape2D {
    ShapeHandle::new(CollisionRect::new(self.half_extents()))
  }
  fn collision_group(&self) -> CollisionGroups {
    CollisionGroups::new()
  }
  fn coltype(&self) -> CollidableType {
    CollidableType::Generic
  }
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
  Generic, // When the type doesn't really matter
}

pub trait GameObjRegistrar<T>
where
  T: Collidable + ?Sized,
{
  fn register(&mut self, register_me: &T, dat: CollidableDat) -> CollisionObjectHandle;
  fn register_with_group(
    &mut self,
    register_me: &T,
    group: CollisionGroups,
    dat: CollidableDat,
  ) -> CollisionObjectHandle;
}

impl<T: Collidable + ?Sized> GameObjRegistrar<T> for CollW {
  fn register(&mut self, register_me: &T, dat: CollidableDat) -> CollisionObjectHandle {
    self.register_with_group(register_me, register_me.collision_group(), dat)
  }

  fn register_with_group(
    &mut self,
    register_me: &T,
    group: CollisionGroups,
    dat: CollidableDat,
  ) -> CollisionObjectHandle {
    let q = nc::world::GeometricQueryType::Contacts(0.0, 0.0);
    self.add(register_me.location(), register_me.shape(), group, q, dat).handle()
  }
}

struct SameEntityFilter;

impl BroadPhasePairFilter<Meters, CollidableDat> for SameEntityFilter {
  fn is_pair_valid(
    &self,
    b1: &CollisionObject<Meters, CollidableDat>,
    b2: &CollisionObject<Meters, CollidableDat>,
  ) -> bool {
    // Disable self-collision. Compound rooms for example are composed of multiple collidables
    // that might touch, with the same id.
    if b1.data().id == b2.data().id {
      return false;
    };
    true
  }
}

pub struct CollGroups;

/// The syntax is { fn_name [member,ship] [white,list] [black,list] } where the white and black
/// lists are optional
macro_rules! new_coll_grp {
  { $name:ident [$( $m:expr ),+] } => {
      pub fn $name () -> CollisionGroups {
        let mut cg = CollisionGroups::new();
        cg.set_membership(&[$($m,)*]);
        cg
      }
  };
  { $name:ident [$( $m:expr ),+] [$( $w:expr ),+] } => {
      pub fn $name () -> CollisionGroups {
        let mut cg = CollisionGroups::new();
        cg.set_membership(&[$($m,)*]);
        cg.set_whitelist(&[$($w,)*]);
        cg
      }
  };
  { $name:ident [$( $m:expr ),+] [$( $w:expr ),*] [$( $b:expr ),*] } => {
      pub fn $name () -> CollisionGroups {
        let mut cg = CollisionGroups::new();
        cg.set_membership(&[$($m,)*]);
        cg.set_whitelist(&[$($w,)*]);
        cg.set_blacklist(&[$($b,)*]);
        cg
      }
  };
}

impl CollGroups {
  new_coll_grp! { wall_cg [1] }
}

#[cfg(test)]
mod test {
  use super::*;

  new_coll_grp! { test_cg [1, 2, 3] }
  new_coll_grp! { test_cg2 [1, 2, 3] [1,2] }
  new_coll_grp! { test_cg3 [1, 2, 3] [1,2] [3,4] }

  #[test]
  fn test_group_gen() {
    let tcg = test_cg();
    assert!(tcg.is_member_of(1));
    assert!(tcg.is_member_of(2));
    assert!(tcg.is_member_of(3));
    assert!(!tcg.is_member_of(4));
    assert!(tcg.is_group_whitelisted(1));
    let tcg2 = test_cg2();
    assert!(tcg2.is_group_whitelisted(1));
    assert!(tcg2.is_group_whitelisted(2));
    assert!(!tcg2.is_group_whitelisted(3));
    let tcg3 = test_cg3();
    assert!(tcg3.is_group_blacklisted(3));
    assert!(tcg3.is_group_blacklisted(4));
    assert!(!tcg3.is_group_blacklisted(1));
  }
}
