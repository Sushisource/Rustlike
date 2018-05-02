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
  Generic, // When the type doesn't really matter
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

pub struct CollGroups;

/// The syntax is { fn_name [member,ship] [white,list] [black,list] } where the white and black
/// lists are optional
macro_rules! new_coll_grp {
  { $name:ident [$( $m:expr ),+] } => {
    new_coll_grp! { $name [$($m),*] [] [] }
  };
  { $name:ident [$( $m:expr ),+] [$( $w:expr ),+] } => {
    new_coll_grp! { $name [$($m),*] [$($w),*] [] }
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
