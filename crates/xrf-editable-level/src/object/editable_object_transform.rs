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
  /// The world transform, `CCustomObject::OnUpdateTransform`: scaled first, then turned by `setXYZi` of the negated
  /// angles (heading `y`, pitch `x`, bank `z`), then moved, composed with `Fmatrix::mul` in the editor's order.
  pub fn to_matrix(self) -> Matrix4x4 {
    let rotation: Affine3d = Affine3d::from_xyz_inverse(&-&self.rotation);

    Matrix4x4::from_translation(&self.position)
      .multiply(&Matrix4x4::from(&rotation))
      .multiply(&Matrix4x4::from_scale(&self.scale))
  }

  /// The position, rotation and scale [`Self::to_matrix`] builds a matrix from: each axis row's length is its scale, a
  /// mirroring matrix taking all three negative, and `Fmatrix::getHPB` of the rows divided by them gives the angles.
  ///
  /// # Errors
  ///
  /// Returns an error when the matrix is not affine (its fourth column is not `0, 0, 0, 1`) or collapses an axis.
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

    Ok(Self {
      position: affine.translation,
      rotation: -&rotation.get_xyz_inverse(),
      scale,
    })
  }

  /// The saved transform composing into `matrix`'s very bits: [`Self::from_matrix`], then [`Self::find_saved`] for
  /// one whose [`Self::to_matrix`] is `matrix`, else the decomposed transform rounded as saved, as for a matrix the
  /// editor composed from other angles (a pitch straight up or down) or no saved text gives.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::from_matrix`] does.
  pub fn from_matrix_as_saved(matrix: &Matrix4x4) -> XrfResult<Self> {
    let decomposed: Self = Self::from_matrix(matrix)?;
    let bits: [u32; 16] = matrix.values.map(f32::to_bits);

    Ok(
      decomposed
        .find_saved(|transform| transform.to_matrix().values.map(f32::to_bits) == bits)
        .unwrap_or_else(|| decomposed.round_as_saved()),
    )
  }

  /// The first saved transform near this one that `accepts`: this one rounded as saved, then each angle and scale a
  /// `%f` step or two either way (`EditableLevelValue::get_saved_neighbours`). A transform decomposed from what the
  /// editor computed off its saved text lands a few units in the last place off that text, which can be a whole step
  /// (`16.000002` for a scale of 16); the position comes back exactly and is only rounded.
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
