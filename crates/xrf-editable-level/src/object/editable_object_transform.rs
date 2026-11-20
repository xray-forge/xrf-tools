use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::{Affine3d, Matrix4x4, Vector3d};

use crate::level::editable_level_value::EditableLevelValue;

/// Where a scene object stands, as `CCustomObject` keeps it: a position, rotation angles about `x`, `y` and `z` in
/// radians, and a scale along each axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableObjectTransform {
  pub position: Vector3d,
  pub rotation: Vector3d,
  pub scale: Vector3d,
}

impl Default for EditableObjectTransform {
  fn default() -> Self {
    Self {
      position: Vector3d::ZERO,
      rotation: Vector3d::ZERO,
      scale: Vector3d::ONE,
    }
  }
}

impl EditableObjectTransform {
  /// The largest cosine between two axes still taken as square: above single-precision rotation noise (about `1e-6`),
  /// below the shear an uneven scale composed with a turn leaves.
  const SHEAR_TOLERANCE: f32 = 1e-4;

  /// The world transform, `CCustomObject::OnUpdateTransform`: scaled first, then turned by `setXYZi` of the negated
  /// angles (heading `y`, pitch `x`, bank `z`), then moved, composed with `Fmatrix::mul` in the editor's order.
  pub fn to_matrix(self) -> Matrix4x4 {
    let rotation: Affine3d = Affine3d::from_xyz_inverse(&-&self.rotation);

    Matrix4x4::from_translation(&self.position)
      .multiply(&Matrix4x4::from(&rotation))
      .multiply(&Matrix4x4::from_scale(&self.scale))
  }

  /// The transform [`Self::to_matrix`] builds `matrix` from, its angles by `Fmatrix::getHPB`. A mirroring matrix takes
  /// all three scales negative.
  ///
  /// # Errors
  ///
  /// Returns an error when the matrix is not affine, collapses an axis, or shears its axes: a scene object scales along
  /// its own axes before it turns, so no transform composes into a sheared matrix.
  pub fn from_matrix(matrix: &Matrix4x4) -> XrfResult<Self> {
    if matrix.get_fourth_column() != Matrix4x4::AFFINE_COLUMN {
      return Err(XrfError::new_invalid_error(format!(
        "Transform's fourth column {:?} is not an affine one",
        matrix.get_fourth_column()
      )));
    }

    let affine: Affine3d = matrix.to_affine();
    let sign: f32 = if affine.x_axis.cross(&affine.y_axis).dot(&affine.z_axis) < 0.0 {
      -1.0
    } else {
      1.0
    };
    let scale: Vector3d = Vector3d::new(
      sign * affine.x_axis.length(),
      sign * affine.y_axis.length(),
      sign * affine.z_axis.length(),
    );

    if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
      return Err(XrfError::new_invalid_error(format!(
        "Transform collapses an axis, scale {scale:?}"
      )));
    }

    let rotation: Affine3d = Affine3d {
      x_axis: &affine.x_axis / scale.x,
      y_axis: &affine.y_axis / scale.y,
      z_axis: &affine.z_axis / scale.z,
      translation: Vector3d::ZERO,
    };
    let cosines: Vector3d = Vector3d::new(
      rotation.y_axis.dot(&rotation.z_axis),
      rotation.z_axis.dot(&rotation.x_axis),
      rotation.x_axis.dot(&rotation.y_axis),
    );

    if cosines
      .to_array()
      .iter()
      .any(|cosine| cosine.abs() > Self::SHEAR_TOLERANCE)
    {
      return Err(XrfError::new_invalid_error(format!(
        "Transform shears its axes, cosines {cosines:?} between them; no scene object stands at it"
      )));
    }

    Ok(Self {
      position: affine.translation,
      rotation: -&rotation.get_xyz_inverse(),
      scale,
    })
  }

  /// The saved transform among [`Self::find_saved`]'s neighbours that composes into `matrix` bit for bit. A matrix
  /// composed from other angles (a pitch straight up or down) or from no saved text gives the decomposed transform
  /// rounded as saved.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::from_matrix`] does.
  pub fn from_matrix_as_saved(matrix: &Matrix4x4) -> XrfResult<Self> {
    let decomposed: Self = Self::from_matrix(matrix)?;

    Ok(
      decomposed
        .find_composing(matrix)
        .unwrap_or_else(|| decomposed.round_as_saved()),
    )
  }

  /// [`Self::find_saved`] accepting only a bit-exact [`Self::to_matrix`], in a fraction of the compositions. An axis's
  /// composed row depends on its own scale alone (`Fmatrix::mul`), so each scale is matched separately under each
  /// rotation, and the whole is composed once to confirm.
  fn find_composing(&self, matrix: &Matrix4x4) -> Option<Self> {
    let rounded: Self = self.round_as_saved();
    let bits: [u32; 16] = matrix.values.map(f32::to_bits);
    let moved: Matrix4x4 = Matrix4x4::from_translation(&rounded.position);
    let scales: [[f32; 5]; 3] = rounded.scale.to_array().map(EditableLevelValue::get_saved_neighbours);

    Self::get_saved_neighbours(&rounded.rotation)
      .into_iter()
      .find_map(|rotation| {
        let turned: Matrix4x4 = moved.multiply(&Matrix4x4::from(&Affine3d::from_xyz_inverse(&-&rotation)));
        let mut scale: [f32; 3] = [1.0; 3];

        for axis in 0..3 {
          scale[axis] = *scales[axis].iter().find(|candidate| {
            let mut along: [f32; 3] = [1.0; 3];

            along[axis] = **candidate;

            let row: [f32; 16] = turned.multiply(&Matrix4x4::from_scale(&Vector3d::from(along))).values;

            (axis * 4..axis * 4 + 4).all(|index| row[index].to_bits() == bits[index])
          })?;
        }

        let transform: Self = Self {
          position: rounded.position,
          rotation,
          scale: Vector3d::from(scale),
        };

        (transform.to_matrix().values.map(f32::to_bits) == bits).then_some(transform)
      })
  }

  /// The first saved transform near this one that `accepts`: this one rounded as saved, then each angle and scale a
  /// `%f` step or two either way. A transform decomposed from what the editor computed can be a whole step off its text
  /// (`16.000002` for a scale of 16); the position comes back exact and is only rounded.
  pub fn find_saved(&self, accepts: impl Fn(&Self) -> bool) -> Option<Self> {
    let rounded: Self = self.round_as_saved();
    let scales: Vec<Vector3d> = Self::get_saved_neighbours(&rounded.scale);

    Self::get_saved_neighbours(&rounded.rotation)
      .into_iter()
      .find_map(|rotation| {
        scales.iter().find_map(|scale| {
          let transform: Self = Self {
            position: rounded.position,
            rotation,
            scale: *scale,
          };

          accepts(&transform).then_some(transform)
        })
      })
  }

  /// Every vector of each component's saved neighbours, the vector itself first.
  fn get_saved_neighbours(vector: &Vector3d) -> Vec<Vector3d> {
    let [x, y, z] = vector.to_array().map(EditableLevelValue::get_saved_neighbours);
    let mut vectors: Vec<Vector3d> = Vec::with_capacity(x.len() * y.len() * z.len());

    for x in x {
      for y in y {
        for z in z {
          vectors.push(Vector3d::new(x, y, z));
        }
      }
    }

    vectors
  }

  /// The transform as it reads back once saved, each component rounded by [`EditableLevelValue::round_as_saved`].
  pub fn round_as_saved(&self) -> Self {
    Self {
      position: EditableLevelValue::round_vector3_as_saved(&self.position),
      rotation: EditableLevelValue::round_vector3_as_saved(&self.rotation),
      scale: EditableLevelValue::round_vector3_as_saved(&self.scale),
    }
  }

  /// # Errors
  ///
  /// Returns an error naming the field that is missing or not three floats.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      position: EditableLevelValue::read_vector3(section, "position")?,
      rotation: EditableLevelValue::read_vector3(section, "rotation")?,
      scale: EditableLevelValue::read_vector3(section, "scale")?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("position", EditableLevelValue::format_vector3(&self.position));
    section.set("rotation", EditableLevelValue::format_vector3(&self.rotation));
    section.set("scale", EditableLevelValue::format_vector3(&self.scale));
  }
}
