use xrf_db::{OgfBone, OgfBoneIkData, Vector3d};

use crate::data::visual_description::VisualBone;
use crate::pack::visual_conversion::convert_vector;

/// A bone's bind transform: a rotation basis in row-vector order, and a translation.
///
/// Mirrors `Fmatrix`'s 4x3 use, whose rows `i`, `j`, `k` are the basis and `c` the translation, because that is the
/// layout every formula below is copied from. A point is transformed as `p * R + c`, not `R * p`.
#[derive(Clone, Debug)]
struct BindTransform {
  i: Vector3d,
  j: Vector3d,
  k: Vector3d,
  c: Vector3d,
}

impl BindTransform {
  /// The transform a root bone composes against, which is what the engine walks down from.
  fn identity() -> Self {
    Self {
      i: Vector3d { x: 1.0, y: 0.0, z: 0.0 },
      j: Vector3d { x: 0.0, y: 1.0, z: 0.0 },
      k: Vector3d { x: 0.0, y: 0.0, z: 1.0 },
      c: Vector3d { x: 0.0, y: 0.0, z: 0.0 },
    }
  }

  /// One bone's bind transform, exactly as the engine composes it.
  ///
  /// `SkeletonCustom.cpp:306` does `setXYZi(bind_rotation)` then `translate_over(bind_position)`, and
  /// `setXYZi(x, y, z)` is `setHPB(-y, -x, -z)` (`_matrix.h:504`). `translate_over` overwrites the translation rather
  /// than accumulating, which is why the rotation is built first and the position simply assigned.
  fn from_bind(rotation: &Vector3d, position: &Vector3d) -> Self {
    // The engine's own argument order, kept verbatim so this reads against `_matrix.h` rather than against intuition.
    let (h, p, b) = (-rotation.y, -rotation.x, -rotation.z);

    let (sh, ch) = (h.sin(), h.cos());
    let (sp, cp) = (p.sin(), p.cos());
    let (sb, cb) = (b.sin(), b.cos());

    let (cc, cs, sc, ss) = (ch * cb, ch * sb, sh * cb, sh * sb);

    Self {
      i: Vector3d {
        x: cc - sp * ss,
        y: -cp * sb,
        z: sp * cs + sc,
      },
      j: Vector3d {
        x: sp * sc + cs,
        y: cp * cb,
        z: ss - sp * cc,
      },
      k: Vector3d {
        x: -cp * sh,
        y: sp,
        z: cp * ch,
      },
      c: position.clone(),
    }
  }

  /// This transform followed by `parent`, which is `Fmatrix::mul_43(parent, self)`.
  ///
  /// Written out rather than expressed as a generic matrix product because the operand order is the thing that goes
  /// wrong: `mul_43(A, B)` composes so that a point passes through `B` first (`matrix.cpp:120`).
  fn then(&self, parent: &Self) -> Self {
    Self {
      i: parent.rotate(&self.i),
      j: parent.rotate(&self.j),
      k: parent.rotate(&self.k),
      c: parent.transform(&self.c),
    }
  }

  fn rotate(&self, vector: &Vector3d) -> Vector3d {
    Vector3d {
      x: self.i.x * vector.x + self.j.x * vector.y + self.k.x * vector.z,
      y: self.i.y * vector.x + self.j.y * vector.y + self.k.y * vector.z,
      z: self.i.z * vector.x + self.j.z * vector.y + self.k.z * vector.z,
    }
  }

  fn transform(&self, point: &Vector3d) -> Vector3d {
    let rotated: Vector3d = self.rotate(point);

    Vector3d {
      x: rotated.x + self.c.x,
      y: rotated.y + self.c.y,
      z: rotated.z + self.c.z,
    }
  }
}

/// Converts a bone list into the renderer-facing skeleton, resolving the bind pose when the file carries one.
///
/// Only each joint's world position and its parent's index survive: that is what draws a skeleton, and it keeps the
/// matrix work - which is where a sign or an operand order goes wrong invisibly - on this side of the wire. Positions
/// pass through the same conversion as the mesh, so the skeleton lands in the geometry rather than mirrored beside it.
///
/// A visual with no IK chunk still reports its bone names and parents, with no positions: the hierarchy is worth
/// listing even when nothing can be drawn from it.
pub fn convert_bones(bones: &[OgfBone], ik_data: Option<&[OgfBoneIkData]>) -> Vec<VisualBone> {
  // The engine reads one IK record per bone, in bone order (`SkeletonCustom.cpp:297`), so a chunk of a different
  // length is not a chunk this pairing understands.
  let binds: Option<&[OgfBoneIkData]> = ik_data.filter(|it| it.len() == bones.len());
  let world: Vec<Option<BindTransform>> = match binds {
    Some(binds) => resolve_world_transforms(bones, binds),
    None => vec![None; bones.len()],
  };

  bones
    .iter()
    .enumerate()
    .map(|(index, bone)| VisualBone {
      name: bone.name.clone(),
      parent: bone.parent.clone(),
      parent_index: find_parent(bones, &bone.parent).map(|it| it as u32),
      bind_position: world[index].as_ref().map(|transform| convert_vector(&transform.c)),
    })
    .collect()
}

/// Every bone's world bind transform, or `None` for a bone whose parent chain does not reach a root.
///
/// Resolved by repeated passes rather than by recursion: the format does not promise parents precede children, and a
/// chain that refers to itself must terminate rather than overflow the stack. Each pass places every bone whose parent
/// is already placed, so a pass that places nothing means the rest is unreachable.
fn resolve_world_transforms(bones: &[OgfBone], binds: &[OgfBoneIkData]) -> Vec<Option<BindTransform>> {
  let local: Vec<BindTransform> = binds
    .iter()
    .map(|it| BindTransform::from_bind(&it.bind_rotation, &it.bind_position))
    .collect();
  let parents: Vec<Option<usize>> = bones.iter().map(|it| find_parent(bones, &it.parent)).collect();

  let mut world: Vec<Option<BindTransform>> = vec![None; bones.len()];

  loop {
    let mut placed: usize = 0;

    for index in 0..bones.len() {
      if world[index].is_some() {
        continue;
      }

      let resolved: Option<BindTransform> = match parents[index] {
        // A root bone's parent is the identity, which is what the engine walks down from.
        None => Some(local[index].then(&BindTransform::identity())),
        Some(parent) => world[parent].as_ref().map(|it| local[index].then(it)),
      };

      if let Some(resolved) = resolved {
        world[index] = Some(resolved);
        placed += 1;
      }
    }

    if placed == 0 {
      return world;
    }
  }
}

/// The index of the bone a name refers to, or `None` for a root or a name no bone carries.
fn find_parent(bones: &[OgfBone], parent: &str) -> Option<usize> {
  if parent.is_empty() {
    return None;
  }

  bones.iter().position(|it| it.name == parent)
}

#[cfg(test)]
mod tests {
  use std::f32::consts::FRAC_PI_2;

  use super::{BindTransform, Vector3d};

  fn vector(x: f32, y: f32, z: f32) -> Vector3d {
    Vector3d { x, y, z }
  }

  fn assert_close(actual: &Vector3d, expected: Vector3d) {
    let epsilon: f32 = 1e-5;

    assert!(
      (actual.x - expected.x).abs() < epsilon
        && (actual.y - expected.y).abs() < epsilon
        && (actual.z - expected.z).abs() < epsilon,
      "expected {expected:?}, got {actual:?}"
    );
  }

  #[test]
  fn an_unrotated_bind_is_a_plain_translation() {
    let transform: BindTransform = BindTransform::from_bind(&vector(0.0, 0.0, 0.0), &vector(1.0, 2.0, 3.0));

    assert_close(&transform.i, vector(1.0, 0.0, 0.0));
    assert_close(&transform.j, vector(0.0, 1.0, 0.0));
    assert_close(&transform.k, vector(0.0, 0.0, 1.0));
    assert_close(&transform.c, vector(1.0, 2.0, 3.0));
  }

  #[test]
  fn a_quarter_turn_about_y_follows_the_engines_euler_convention() {
    // `setXYZi(x, y, z)` is `setHPB(-y, -x, -z)`, so a quarter turn about y enters `setHPB` as h = -pi/2. Worked
    // through `matrix.cpp:437` by hand, that basis takes +x to -z. Getting the sign or the argument order wrong still
    // produces a rotation, just the wrong one, which is why this asserts the basis rather than a round trip.
    let transform: BindTransform = BindTransform::from_bind(&vector(0.0, FRAC_PI_2, 0.0), &vector(0.0, 0.0, 0.0));

    assert_close(&transform.i, vector(0.0, 0.0, -1.0));
    assert_close(&transform.j, vector(0.0, 1.0, 0.0));
    assert_close(&transform.k, vector(1.0, 0.0, 0.0));
    assert_close(&transform.transform(&vector(1.0, 0.0, 0.0)), vector(0.0, 0.0, -1.0));
  }

  #[test]
  fn a_child_bind_is_composed_through_its_parent() {
    // `mul_43(parent, child)` passes a point through the child first. Composing the other way round would put this
    // joint at (1, 0, 0) instead, which is a plausible looking skeleton in the wrong pose.
    let parent: BindTransform = BindTransform::from_bind(&vector(0.0, FRAC_PI_2, 0.0), &vector(0.0, 0.0, 0.0));
    let child: BindTransform = BindTransform::from_bind(&vector(0.0, 0.0, 0.0), &vector(1.0, 0.0, 0.0));

    assert_close(&child.then(&parent).c, vector(0.0, 0.0, -1.0));
  }

  #[test]
  fn translations_accumulate_down_a_chain() {
    let root: BindTransform = BindTransform::from_bind(&vector(0.0, 0.0, 0.0), &vector(1.0, 2.0, 3.0));
    let child: BindTransform = BindTransform::from_bind(&vector(0.0, 0.0, 0.0), &vector(0.0, 5.0, 0.0));

    assert_close(&child.then(&root).c, vector(1.0, 7.0, 3.0));
  }
}
