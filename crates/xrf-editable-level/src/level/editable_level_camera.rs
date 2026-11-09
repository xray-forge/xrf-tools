use xrf_error::XrfResult;
use xrf_ini::IniSection;
use xrf_math::Vector3d;

use crate::level::editable_level_value::EditableLevelValue;

/// Where the editor's camera stood when the scene was saved, the scene's `[camera]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableLevelCamera {
  /// `pos`.
  pub position: Vector3d,
  /// `hpb`: heading, pitch and bank, in radians.
  pub rotation: Vector3d,
}

/// A camera at the origin, unturned.
impl Default for EditableLevelCamera {
  fn default() -> Self {
    Self {
      position: Vector3d::ZERO,
      rotation: Vector3d::ZERO,
    }
  }
}

impl EditableLevelCamera {
  pub const SECTION: &str = "camera";

  /// # Errors
  ///
  /// Returns an error when `pos` or `hpb` is missing or not three floats.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      position: EditableLevelValue::read_vector3(section, "pos")?,
      rotation: EditableLevelValue::read_vector3(section, "hpb")?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("pos", EditableLevelValue::format_vector3(&self.position));
    section.set("hpb", EditableLevelValue::format_vector3(&self.rotation));
  }
}
