use xrf_db::{Quaternion, Vector3d};

use crate::data::visual_description::VisualTransform;

/// A bone's transform: a rotation basis in row-vector order, and a translation.
///
/// Mirrors `Fmatrix`'s 4x3 use, whose rows `i`, `j`, `k` are the basis and `c` the translation, because that is the
/// layout every formula below is copied from. A point is transformed as `p * R + c`, not `R * p`.
///
/// One type for the bind pose and for an animated pose, because the engine composes both the same way: an animated
/// bone's local transform replaces its bind transform rather than multiplying it (`SkeletonAnimated.cpp:914`), and
/// both go through `mul_43(parent, local)`.
#[derive(Clone, Debug)]
pub(crate) struct BindTransform {
  pub(crate) i: Vector3d,
  pub(crate) j: Vector3d,
  pub(crate) k: Vector3d,
  pub(crate) c: Vector3d,
}

impl BindTransform {
  /// The transform a root bone composes against, which is what the engine walks down from.
  pub(crate) fn identity() -> Self {
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
  pub(crate) fn from_bind(rotation: &Vector3d, position: &Vector3d) -> Self {
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

  /// One bone's animated transform, from a motion key.
  ///
  /// `Fmatrix::mk_xform` verbatim (`matrix.cpp:41`), whose rows are the basis in the same order [`Self::from_bind`]
  /// builds them. An animated key carries the whole local transform, so nothing of the bind pose enters here.
  pub(crate) fn from_key(rotation: &Quaternion, translation: &Vector3d) -> Self {
    let (x, y, z, w) = (rotation.x, rotation.y, rotation.z, rotation.w);

    let (xx, yy, zz) = (x * x, y * y, z * z);
    let (xy, xz, yz) = (x * y, x * z, y * z);
    let (wx, wy, wz) = (w * x, w * y, w * z);

    Self {
      i: Vector3d {
        x: 1.0 - 2.0 * (yy + zz),
        y: 2.0 * (xy - wz),
        z: 2.0 * (xz + wy),
      },
      j: Vector3d {
        x: 2.0 * (xy + wz),
        y: 1.0 - 2.0 * (xx + zz),
        z: 2.0 * (yz - wx),
      },
      k: Vector3d {
        x: 2.0 * (xz - wy),
        y: 2.0 * (yz + wx),
        z: 1.0 - 2.0 * (xx + yy),
      },
      c: translation.clone(),
    }
  }

  /// This transform followed by `parent`, which is `Fmatrix::mul_43(parent, self)`.
  ///
  /// Written out rather than expressed as a generic matrix product because the operand order is the thing that goes
  /// wrong: `mul_43(A, B)` composes so that a point passes through `B` first (`matrix.cpp:120`).
  pub(crate) fn then(&self, parent: &Self) -> Self {
    Self {
      i: parent.rotate(&self.i),
      j: parent.rotate(&self.j),
      k: parent.rotate(&self.k),
      c: parent.transform(&self.c),
    }
  }

  /// The same transform expressed in the renderer's mirrored space.
  ///
  /// Geometry reaches the renderer with Z negated (`convert_vector`), so a transform that is to act on it has to be
  /// mirrored too - and mirroring a transform is not mirroring its parts. Conjugating by `S = diag(1, 1, -1)` gives
  /// `S M S`, which negates the z of the first two basis vectors, the x and y of the third, and the z of the
  /// translation. Negating every z instead leaves a rotation that turns the wrong way about x and y, which looks
  /// plausible on a symmetric pose and wrong on every other frame.
  pub(crate) fn mirrored(&self) -> Self {
    Self {
      i: Vector3d {
        x: self.i.x,
        y: self.i.y,
        z: -self.i.z,
      },
      j: Vector3d {
        x: self.j.x,
        y: self.j.y,
        z: -self.j.z,
      },
      k: Vector3d {
        x: -self.k.x,
        y: -self.k.y,
        z: self.k.z,
      },
      c: Vector3d {
        x: self.c.x,
        y: self.c.y,
        z: -self.c.z,
      },
    }
  }

  /// The mirrored transform as the wire carries it, which is the only form that leaves this crate.
  ///
  /// Mirroring happens here rather than at the call sites so a transform cannot reach a renderer in engine space:
  /// there is one way out, and it converts.
  pub(crate) fn to_renderer_space(&self) -> VisualTransform {
    let mirrored: Self = self.mirrored();

    VisualTransform {
      i: mirrored.i,
      j: mirrored.j,
      k: mirrored.k,
      c: mirrored.c,
    }
  }

  pub(crate) fn rotate(&self, vector: &Vector3d) -> Vector3d {
    Vector3d {
      x: self.i.x * vector.x + self.j.x * vector.y + self.k.x * vector.z,
      y: self.i.y * vector.x + self.j.y * vector.y + self.k.y * vector.z,
      z: self.i.z * vector.x + self.j.z * vector.y + self.k.z * vector.z,
    }
  }

  pub(crate) fn transform(&self, point: &Vector3d) -> Vector3d {
    let rotated: Vector3d = self.rotate(point);

    Vector3d {
      x: rotated.x + self.c.x,
      y: rotated.y + self.c.y,
      z: rotated.z + self.c.z,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::f32::consts::FRAC_PI_2;

  use xrf_db::Quaternion;

  use super::{BindTransform, Vector3d};
  use crate::pack::visual_conversion::convert_vector;

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

  #[test]
  fn an_identity_quaternion_key_is_a_plain_translation() {
    let transform: BindTransform = BindTransform::from_key(
      &Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
      },
      &vector(1.0, 2.0, 3.0),
    );

    assert_close(&transform.i, vector(1.0, 0.0, 0.0));
    assert_close(&transform.j, vector(0.0, 1.0, 0.0));
    assert_close(&transform.k, vector(0.0, 0.0, 1.0));
    assert_close(&transform.c, vector(1.0, 2.0, 3.0));
  }

  #[test]
  fn a_quarter_turn_key_about_y_builds_the_basis_mk_xform_states() {
    // Worked through `matrix.cpp:41` by hand for q = (0, sin45, 0, cos45): i = (0, 0, 1), j = (0, 1, 0),
    // k = (-1, 0, 0), so +x goes to +z. Asserting the whole basis rather than one mapped vector is what would catch a
    // transposition, which is still a valid rotation and so survives every numeric invariant.
    //
    // Deliberately the opposite direction to the euler test above, and not a contradiction: `setXYZi` negates its
    // arguments (`setHPB(-y, -x, -z)`), so `setXYZi(0, pi/2, 0)` turns the other way. Expecting the two paths to agree
    // for the same nominal angle is the trap here.
    let half: f32 = FRAC_PI_2 / 2.0;
    let transform: BindTransform = BindTransform::from_key(
      &Quaternion {
        x: 0.0,
        y: half.sin(),
        z: 0.0,
        w: half.cos(),
      },
      &vector(0.0, 0.0, 0.0),
    );

    assert_close(&transform.i, vector(0.0, 0.0, 1.0));
    assert_close(&transform.j, vector(0.0, 1.0, 0.0));
    assert_close(&transform.k, vector(-1.0, 0.0, 0.0));
    assert_close(&transform.transform(&vector(1.0, 0.0, 0.0)), vector(0.0, 0.0, 1.0));
  }

  #[test]
  fn mirroring_a_transform_agrees_with_mirroring_what_it_produces() {
    // The property that makes a mirrored transform usable on mirrored geometry: posing a point and then converting it
    // must land where converting the point and then posing it with the mirrored transform does. An asymmetric rotation
    // and an off-axis translation, because a rotation about y alone survives the wrong mirror unnoticed.
    let transform: BindTransform = BindTransform::from_bind(&vector(0.3, -0.7, 1.1), &vector(1.0, -2.0, 3.0));
    let point: Vector3d = vector(0.4, 0.5, -0.6);

    let posed_then_converted: Vector3d = convert_vector(&transform.transform(&point));
    let converted_then_posed: Vector3d = transform.mirrored().transform(&convert_vector(&point));

    assert_close(&converted_then_posed, posed_then_converted);
  }

  #[test]
  fn mirroring_twice_returns_the_transform() {
    let transform: BindTransform = BindTransform::from_bind(&vector(0.3, -0.7, 1.1), &vector(1.0, -2.0, 3.0));
    let round_trip: BindTransform = transform.mirrored().mirrored();

    assert_close(&round_trip.i, transform.i.clone());
    assert_close(&round_trip.j, transform.j.clone());
    assert_close(&round_trip.k, transform.k.clone());
    assert_close(&round_trip.c, transform.c.clone());
  }
}
