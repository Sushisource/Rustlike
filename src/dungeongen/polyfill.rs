// Taken from: https://gitlab.com/ideasman42/bmesh-rs/blob/master/extern/plain_ptr/src/lib.rs
// Licensed: Apache 2.0

/**
 * A simple implementation of the ear cutting algorithm
 * to triangulate simple polygons without holes.
 *
 * \note
 *
 * Changes made for Blender.
 *
 * - loop the array to clip last verts first (less array resizing)
 *
 * - advance the ear to clip each iteration
 *   to avoid fan-filling convex shapes (USE_CLIP_EVEN).
 *
 * - avoid intersection tests when there are no convex points (USE_CONVEX_SKIP).
 *
 * \note
 *
 * No globals - keep threadsafe.
 */

// defines
const USE_CLIP_EVEN: bool = true;
const USE_CONVEX_SKIP: bool = true;
const USE_CLIP_SWEEP: bool = true;
const USE_KDTREE: bool = true;

// TODO, use enum type?
pub type ESign = i8;
pub type Real = f64;

const CONCAVE: i8 = -1;
const TANGENTIAL: i8 = 0;
const CONVEX: i8 = 1;


// ---------------------------------------------------------------------------
// Utility Macro's

macro_rules! elem {
    ($val:expr, $($var:expr), *) => {
        $($val == $var) || *
    }
}

// for when rust gets this hint
macro_rules! unlikely { ($body:expr) => { $body } }
macro_rules! poly_tri_count { ($body:expr) => { ($body) - 2 } }

mod math {
  use super::ESign;
  use super::Real;
  use super::{CONCAVE, TANGENTIAL, CONVEX};

  fn signum_enum(a: Real) -> ESign {
    if a > 0.0 {
      CONVEX
    } else if a < 0.0 {
      CONCAVE
    } else {
      TANGENTIAL
    }
  }

  pub fn area_tri_signed_v2_alt_2x(v1: &[Real; 2], v2: &[Real; 2], v3: &[Real; 2]) -> Real {
    ((v1[0] * (v2[1] - v3[1])) + (v2[0] * (v3[1] - v1[1])) + (v3[0] * (v1[1] - v2[1])))
  }

  pub fn span_tri_v2_sign(v1: &[Real; 2], v2: &[Real; 2], v3: &[Real; 2]) -> ESign {
    return signum_enum(area_tri_signed_v2_alt_2x(v3, v2, v1));
  }

  ///
  /// Scalar cross product of a 2d polygon.
  ///
  /// - equivalent to ``area * 2``
  /// - useful for checking polygon winding (a positive value is clockwise).
  ///
  pub fn cross_poly_v2(coords: &Vec<[Real; 2]>) -> Real {
    // The Trapezium Area Rule
    let mut co_prev = &coords[coords.len() - 1];
    let mut cross: Real = 0.0;
    for co_curr in coords {
      cross += (co_curr[0] - co_prev[0]) * (co_curr[1] + co_prev[1]);
      co_prev = co_curr;
    }
    return cross;
  }
}

mod kdtree2d {
  /**
   * This is a single purpose KDTree
   * based on a more general kdtree with some modifications to better suit polyfill2d.
   *
   *
   * - `KDTreeNode2D` is kept small,
   *   by not storing coords in the nodes and using index values rather then pointers
   *   to reference neg/pos values.
   *
   * - `isect_tri` is the only searching function currently used.
   *   This simply intersects a triangle with the kdtree points.
   *
   * - the KDTree only includes concave points and
   *   isn't used when the polygon is entirely concave.
   */

  use super::Real;
  use super::math;
  use super::CONCAVE;

  pub const KDNODE_UNSET: usize = ::std::u32::MAX as usize;
  const KDNODE_FLAG_REMOVED: u8 = (1 << 0);


  pub struct KDTreeNode2D {
    neg: usize,
    pos: usize,
    index: usize,
    axis: usize,
    /* range is only (0-1) */
    flag: u8,
    parent: usize,
  }

  pub struct KDTree2D<'a> {
    nodes: Vec<KDTreeNode2D>,
    coords: &'a Vec<[super::Real; 2]>,
    root: usize,
    pub totnode: usize,
    nodes_map: Vec<usize>,
    /* index -> node lookup */
  }

  struct KDRange2D {
    min: Real,
    max: Real,
  }

  pub fn new<'a>(
    nodes: Vec<KDTreeNode2D>,
    nodes_map: Vec<usize>,
    tot: usize,
    coords: &'a Vec<[super::Real; 2]>) -> KDTree2D<'a>
  {
    KDTree2D {
      nodes: nodes,
      coords: coords,
      root: KDNODE_UNSET,
      totnode: tot,
      nodes_map: nodes_map,
    }
  }

  pub fn init(tree: &mut KDTree2D, indices: &Vec<super::poly_fill::PolyIndex>) {
    for pi in indices {
      if pi.sign != super::CONVEX {
        tree.nodes.push(
          KDTreeNode2D {
            parent: KDNODE_UNSET,
            // could avoid setting here!
            neg: KDNODE_UNSET,
            pos: KDNODE_UNSET,
            index: pi.index,
            axis: 0,
            flag: 0,
          }
        );
      }
    }
  }

  pub fn balance(tree: &mut KDTree2D) {
    fn balance_recursive(
      nodes: &mut [KDTreeNode2D],
      coords: &Vec<[Real; 2]>,
      axis: usize, ofs: usize,
    ) -> usize
    {
      if nodes.len() == 0 {
        return KDNODE_UNSET;
      } else if nodes.len() == 1 {
        return ofs;
      }

      // quick-sort style sorting around median
      let median = nodes.len() / 2;
      {
        let mut neg = 0;
        let mut pos = nodes.len() - 1;

        while pos > neg {
          let co = unsafe { coords[nodes.get_unchecked(pos).index][axis] };
          let mut i = neg.wrapping_sub(1);
          let mut j = pos;

          loop {
            unsafe {
              while coords.get_unchecked(
                nodes.get_unchecked(
                  {
                    i = i.wrapping_add(1);
                    i
                  }).index)[axis] < co {}
              while coords.get_unchecked(
                nodes.get_unchecked(
                  {
                    j = j.wrapping_sub(1);
                    j
                  }).index)[axis] > co && j > neg {}
            }

            if i >= j {
              break;
            }

            nodes.swap(i, j);
          }

          nodes.swap(i, pos);
          if i >= median {
            pos = i - 1;
          }
          if i <= median {
            neg = i + 1;
          }
        }
      }

      // set node and sort subnodes
      let axis_next = (axis + 1) % 2;
      let node_neg = balance_recursive(&mut nodes[..median], coords, axis_next, ofs);
      let node_pos = balance_recursive(&mut nodes[(median + 1)..], coords, axis_next, (median + 1) + ofs);
      {
        let mut node = unsafe { nodes.get_unchecked_mut(median) };
        node.axis = axis;
        node.neg = node_neg;
        node.pos = node_pos;
      }

      return median + ofs;
    }

    tree.root = balance_recursive(&mut tree.nodes[..], tree.coords, 0, 0);
  }

  pub fn init_mapping(tree: &mut KDTree2D) {
    let nodes = &mut tree.nodes;
    for i in 0..nodes.len() {
      let index = nodes[i].neg;
      if index != KDNODE_UNSET {
        nodes[index].parent = i;
      }
      let index = nodes[i].pos;
      if index != KDNODE_UNSET {
        nodes[index].parent = i;
      }
      debug_assert!(tree.nodes_map[nodes[i].index] == KDNODE_UNSET);
      tree.nodes_map[nodes[i].index] = i;
    }
    nodes[tree.root].parent = KDNODE_UNSET;
  }

  pub fn node_remove(tree: &mut KDTree2D, index: usize) {
    let mut node_index = tree.nodes_map[index];
    if node_index == KDNODE_UNSET {
      return;
    } else {
      tree.nodes_map[index] = KDNODE_UNSET;
    }

    tree.totnode -= 1;

    debug_assert!((tree.nodes[node_index].flag & KDNODE_FLAG_REMOVED) == 0);
    tree.nodes[node_index].flag |= KDNODE_FLAG_REMOVED;

    let mut node_index_parent;

    while {
      let node = &tree.nodes[node_index];
      node_index_parent = node.parent;

      (node.neg == KDNODE_UNSET) &&
        (node.pos == KDNODE_UNSET) &&
        (node.parent != KDNODE_UNSET)
    } {
      // debug_assert!(&tree.nodes[node_index] as *const _ == node as *const _);
      let node = &mut tree.nodes[node_index_parent];
      if node.neg == node_index {
        node.neg = KDNODE_UNSET;
      } else {
        debug_assert!(node.pos == node_index);
        node.pos = KDNODE_UNSET;
      }
      if (node.flag & KDNODE_FLAG_REMOVED) != 0 {
        node_index = node_index_parent;
      } else {
        break;
      }
    }
  }

  fn isect_tri_recurse(
    tree: &KDTree2D,
    tri_index: &[usize; 3],
    tri_coords: &[&[Real; 2]; 3],
    tri_center: &[Real; 2],
    bounds: &[KDRange2D; 2],
    node: &KDTreeNode2D
  ) -> bool {
    let co = &tree.coords[node.index];

    // bounds then triangle intersect
    if (node.flag & KDNODE_FLAG_REMOVED) == 0 {
      // bounding box test first
      if (co[0] >= bounds[0].min) &&
        (co[0] <= bounds[0].max) &&
        (co[1] >= bounds[1].min) &&
        (co[1] <= bounds[1].max)
        {
          if (math::span_tri_v2_sign(tri_coords[0], tri_coords[1], co) != CONCAVE) &&
            (math::span_tri_v2_sign(tri_coords[1], tri_coords[2], co) != CONCAVE) &&
            (math::span_tri_v2_sign(tri_coords[2], tri_coords[0], co) != CONCAVE)
            {
              if !elem!(node.index, tri_index[0], tri_index[1], tri_index[2]) {
                return true;
              }
            }
        }
    }

    macro_rules! isect_tri_recurse_neg {
            () => {
            ((node.neg != KDNODE_UNSET) && (co[node.axis] > bounds[node.axis].min)) &&
             (isect_tri_recurse(tree, tri_index, tri_coords, tri_center, bounds,
                                &tree.nodes[node.neg]))
        }}
    macro_rules! isect_tri_recurse_pos {
            () => {
            ((node.pos != KDNODE_UNSET) && (co[node.axis] < bounds[node.axis].max)) &&
             (isect_tri_recurse(tree, tri_index, tri_coords, tri_center, bounds,
                                &tree.nodes[node.pos]))
        }}

    if tri_center[node.axis] > co[node.axis] {
      if isect_tri_recurse_pos!() {
        return true;
      }
      if isect_tri_recurse_neg!() {
        return true;
      }
    } else {
      if isect_tri_recurse_neg!() {
        return true;
      }
      if isect_tri_recurse_pos!() {
        return true;
      }
    }
    debug_assert!(node.index != KDNODE_UNSET);

    return false;
  }

  pub fn isect_tri(
    tree: &KDTree2D,
    ind: &[usize; 3]
  ) -> bool
  {
    let vs = [
      &tree.coords[ind[0]],
      &tree.coords[ind[1]],
      &tree.coords[ind[2]],
    ];

    let bounds: [KDRange2D; 2] = [
      KDRange2D {
        min: vs[0][0].min(vs[1][0]).min(vs[2][0]),
        max: vs[0][0].max(vs[1][0]).max(vs[2][0]),
      },
      KDRange2D {
        min: vs[0][1].min(vs[1][1]).min(vs[2][1]),
        max: vs[0][1].max(vs[1][1]).max(vs[2][1]),
      },
    ];

    let tri_center: [Real; 2] = [
      (vs[0][0] + vs[1][0] + vs[2][0]) / 3.0,
      (vs[0][1] + vs[1][1] + vs[2][1]) / 3.0,
    ];

    return isect_tri_recurse(tree, ind, &vs, &tri_center, &bounds, &tree.nodes[tree.root]);
  }
}

mod poly_fill {
  // based on libgdx 2013-11-28, apache 2.0 licensed
  use super::math;
  use super::kdtree2d;
  use super::{ESign, Real};
  use super::{CONCAVE, CONVEX};
  use super::{USE_CONVEX_SKIP, USE_CLIP_SWEEP, USE_CLIP_EVEN, USE_KDTREE};

  pub struct PolyIndex {
    next: usize,
    prev: usize,
    // index into coords
    pub index: usize,
    pub sign: ESign,
  }

  pub struct PolyFill<'a> {
    coords: &'a Vec<[Real; 2]>,
    /// Tracks remaining unclipped coords,
    /// initializes to coords.len(), decrease as ears are clipped.
    coords_tot: usize,

    coords_tot_concave: usize,

    // A polygon with n vertices has a triangulation of n-2 triangles.
    tris: &'a mut Vec<[u32; 3]>,

    // vertex aligned */
    indices: Vec<PolyIndex>,
    indices_first: usize,

    kdtree: kdtree2d::KDTree2D<'a>,
  }

  /**
   * \return CONCAVE, TANGENTIAL or CONVEX
   */
  fn coord_sign_calc(coords: &Vec<[Real; 2]>, indices: &Vec<PolyIndex>, pi: &PolyIndex) -> ESign {
    math::span_tri_v2_sign(
      &coords[indices[pi.prev].index],
      &coords[pi.index],
      &coords[indices[pi.next].index])
  }

  // a version of `coord_sign_calc` that modifies the value in place.
  fn coord_sign_calc_at_poly_index(pf: &mut PolyFill, poly_index: usize) {
    pf.indices[poly_index].sign =
      coord_sign_calc(pf.coords, &pf.indices, &pf.indices[poly_index]);
  }

  fn ear_tip_check(pf: &PolyFill, pi_ear_tip: &PolyIndex) -> bool {
    if USE_CONVEX_SKIP {
      if pf.coords_tot_concave == 0 {
        return true;
      }
    }

    if unlikely!(pi_ear_tip.sign == CONCAVE) {
      return false;
    }

    if USE_KDTREE {
      let ind = [
        pi_ear_tip.index,
        pf.indices[pi_ear_tip.next].index,
        pf.indices[pi_ear_tip.prev].index];

      if kdtree2d::isect_tri(&pf.kdtree, &ind) {
        return false;
      }
    } else {
      let v1 = &pf.coords[pf.indices[pi_ear_tip.prev].index];
      let v2 = &pf.coords[pi_ear_tip.index];
      let v3 = &pf.coords[pf.indices[pi_ear_tip.next].index];

      // Check if any point is inside the triangle formed by previous, current and next vertices.
      // Only consider vertices that are not part of this triangle,
      // or else we'll always find one inside.

      let mut pi_curr = &pf.indices[pf.indices[pi_ear_tip.next].next];
      while pi_curr as *const _ != &pf.indices[pi_ear_tip.prev] as *const _ {
        // Concave vertices can obviously be inside the candidate ear,
        // but so can tangential vertices if they coincide with one of the triangle's vertices.
        if pi_curr.sign != super::CONVEX {
          let v = &pf.coords[pi_curr.index];
          // Because the polygon has clockwise winding order,
          // the area sign will be positive if the point is strictly inside.
          // It will be 0 on the edge, which we want to include as well.

          // note: check (v3, v1) first since it fails _far_ more often
          // then the other 2 checks (those fail equally).
          // It's logical - the chance is low that points exist
          // on the same side as the ear we're clipping off.
          if (math::span_tri_v2_sign(v3, v1, v) != CONCAVE) &&
            (math::span_tri_v2_sign(v1, v2, v) != CONCAVE) &&
            (math::span_tri_v2_sign(v2, v3, v) != CONCAVE)
            {
              return false;
            }
        }

        pi_curr = &pf.indices[pi_curr.next];
      }
    }

    return true;
  }

  fn ear_tip_find(pf: &PolyFill, pi_ear_index_init: usize, reverse: bool) -> usize {
    let mut pi_ear_index = pi_ear_index_init;
    let mut pi_ear = &pf.indices[pi_ear_index];

    let mut i = pf.coords_tot;
    while {
      i -= 1;
      i != 0
    } {
      if ear_tip_check(pf, pi_ear) {
        return pi_ear_index;
      }
      if USE_CLIP_SWEEP {
        pi_ear_index = if reverse { pi_ear.prev } else { pi_ear.next };
      } else {
        pi_ear_index = pi_ear.next;
      }
      pi_ear = &pf.indices[pi_ear_index];
    }

    // Desperate mode: if no vertex is an ear tip,
    // we are dealing with a degenerate polygon (e.g. nearly collinear).
    // Note that the input was not necessarily degenerate,
    // but we could have made it so by clipping some valid ears.
    //
    // Idea taken from Martin Held,
    // "FIST: Fast industrial-strength triangulation of polygons", Algorithmica (1998),
    // http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.115.291
    //
    // Return a convex or tangential vertex if one exists.
    pi_ear_index = pi_ear_index_init;
    pi_ear = &pf.indices[pi_ear_index];
    i = pf.coords_tot;
    while {
      i -= 1;
      i != 0
    } {
      if pi_ear.sign != CONCAVE {
        return pi_ear_index;
      }
      if USE_CLIP_SWEEP {
        pi_ear_index = if reverse { pi_ear.prev } else { pi_ear.next };
      } else {
        pi_ear_index = pi_ear.next;
      }
      pi_ear = &pf.indices[pi_ear_index];
    }

    // If all vertices are concave, just return the last one.
    return pi_ear_index;
  }

  fn coord_remove(pf: &mut PolyFill, pi_index: usize) {
    let next;
    let prev;
    {
      let pi = &pf.indices[pi_index];
      next = pi.next;
      prev = pi.prev;

      if USE_KDTREE {
        // avoid double lookups, since convex coords are ignored when testing intersections
        if pf.kdtree.totnode != 0 {
          kdtree2d::node_remove(&mut pf.kdtree, pi.index);
        }
      }
    }
    pf.indices[next].prev = prev;
    pf.indices[prev].next = next;

    if unlikely!(pf.indices_first == pi_index) {
      let pi = &pf.indices[pi_index];
      pf.indices_first = pi.next;
    }

    pf.coords_tot -= 1;
  }


  fn ear_tip_cut(pf: &mut PolyFill, pi_ear_tip_index: usize) {
    {
      let pi_ear_tip = &pf.indices[pi_ear_tip_index];
      pf.tris.push([pf.indices[pi_ear_tip.prev].index as u32,
        pi_ear_tip.index as u32,
        pf.indices[pi_ear_tip.next].index as u32]);
    }

    coord_remove(pf, pi_ear_tip_index);
  }

  fn triangulate(pf: &mut PolyFill) {
    let mut pi_ear_init_index = pf.indices_first; // USE_CONVEX_SKIP
    let mut reverse = false; // USE_CLIP_SWEEP

    while pf.coords_tot > 3 {
      let pi_ear_index = ear_tip_find(pf, pi_ear_init_index, reverse);


      let pi_prev_index;
      let pi_next_index;

      {
        let pi_ear = &pf.indices[pi_ear_index];
        pi_prev_index = pi_ear.prev;
        pi_next_index = pi_ear.next;
      }

      ear_tip_cut(pf, pi_ear_index);

      let sign_orig_prev;
      let sign_orig_next;
      {
        let pi_prev = &pf.indices[pi_prev_index];
        let pi_next = &pf.indices[pi_next_index];

        sign_orig_prev = pi_prev.sign;
        sign_orig_next = pi_next.sign;

        if USE_CLIP_EVEN {
          if USE_CLIP_SWEEP {
            pi_ear_init_index = if reverse { pi_next.prev } else { pi_prev.next };
          } else {
            pi_ear_init_index = pi_next.next;
          }
        } else {
          pi_ear_init_index = pf.indices_first;
        }
      }

      if sign_orig_prev != CONVEX {
        coord_sign_calc_at_poly_index(pf, pi_prev_index);
        if USE_CONVEX_SKIP {
          if pf.indices[pi_prev_index].sign == CONVEX {
            pf.coords_tot_concave -= 1;
          }
        }
      }

      if sign_orig_next != CONVEX {
        coord_sign_calc_at_poly_index(pf, pi_next_index);
        if USE_CONVEX_SKIP {
          if pf.indices[pi_next_index].sign == CONVEX {
            debug_assert!(pf.coords_tot_concave != 0);
            pf.coords_tot_concave -= 1;
          }
        }
      }

      if USE_CLIP_EVEN {
        let pi_ear_init = &pf.indices[pi_ear_init_index];
        if pi_ear_init.sign != CONVEX {
          // take the extra step since this ear isn't a good candidate
          pi_ear_init_index = if reverse { pi_ear_init.prev } else { pi_ear_init.next };
          reverse = !reverse;
        }
      } else {
        reverse = !reverse;
      }
    }

    if pf.coords_tot == 3 {
      let pi_ear_tip = &pf.indices[pf.indices_first];
      pf.tris.push([
        pf.indices[pi_ear_tip.prev].index as u32,
        pi_ear_tip.index as u32,
        pf.indices[pi_ear_tip.next].index as u32,
      ]);
    }
  }

  pub fn prepare<'a>(
    coords: &'a Vec<[Real; 2]>,
    mut coords_sign: ESign,
    r_tris: &'a mut Vec<[u32; 3]>,
    mut indices: Vec<PolyIndex>,
  ) -> PolyFill<'a>
  {
    if coords_sign == 0 {
      coords_sign = if math::cross_poly_v2(coords) >= 0.0 { 1 } else { -1 };
    }

    // allow int wrapping, correct end-points after
    if coords_sign == 1 {
      for i in 0..coords.len() {
        indices.push(PolyIndex {
          next: i.wrapping_add(1),
          prev: i.wrapping_sub(1),
          index: i,
          sign: 0,
          // dummy
        });
      }
    } else {
      let n: usize = coords.len() - 1;
      for i in 0..coords.len() {
        indices.push(PolyIndex {
          next: i.wrapping_add(1),
          prev: i.wrapping_sub(1),
          index: n - i,
          sign: 0,
          // dummy
        });
      }
    }
    // make circular
    indices[0].prev = coords.len() - 1;
    indices[coords.len() - 1].next = 0;

    let mut coords_tot_concave = 0;
    for i in 0..indices.len() {
      let sign = coord_sign_calc(coords, &indices, &indices[i]);
      indices[i].sign = sign;
      if USE_CONVEX_SKIP {
        if sign != CONVEX {
          coords_tot_concave += 1;
        }
      }
    }

    let kdtree =
      if coords_tot_concave == 0 {
        kdtree2d::new(vec![], vec![], 0, coords)
      } else {
        let mut kdtree = kdtree2d::new(
          Vec::with_capacity(coords_tot_concave),
          vec![kdtree2d::KDNODE_UNSET; coords.len()],
          coords_tot_concave,
          coords);

        kdtree2d::init(&mut kdtree, &indices);
        kdtree2d::balance(&mut kdtree);
        kdtree2d::init_mapping(&mut kdtree);
        kdtree
      };

    PolyFill {
      indices: indices,
      indices_first: 0,

      coords: coords,
      coords_tot: coords.len(),
      coords_tot_concave: coords_tot_concave,

      tris: r_tris,

      // USE_KDTREE
      kdtree: kdtree,
    }
  }

  pub fn calc(pf: &mut PolyFill) {
    triangulate(pf);
  }
}

///
/// This function takes 2D `coords` and fills in `r_tris`, a vector
/// of 3-tuples, which are indexes into `coords`.
///
/// Note that there is no failure state,
/// this is guaranteed to fill in every triangle.
///
pub fn polyfill_calc(coords: &Vec<[Real; 2]>, coords_sign: ESign,
                     r_tris: &mut Vec<[u32; 3]>) {
  let indices: Vec<poly_fill::PolyIndex> = Vec::with_capacity(coords.len());

  let mut pf = poly_fill::prepare(coords, coords_sign, r_tris, indices);

  poly_fill::calc(&mut pf);
}
