use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniSection};
use xrf_math::{Matrix4x4, Vector3d};

use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_shape::{EditableShape, EditableShapeElement};
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A shape object, `CEditShape` as the shape tool places it: an object of `shape.part`, and what a spawn point holds
/// attached to give an entity its volume (a restrictor's, a zone's).
#[derive(Clone, Debug, PartialEq)]
pub struct EditableShapeObject {
  pub object: EditableCustomObject,
  pub shape: EditableShape,
}

impl EditableShapeObject {
  /// As `CEditShape::LoadLTX` reads it from one section.
  ///
  /// # Errors
  ///
  /// Returns an error naming the field that is missing or malformed.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      shape: EditableShape::read_from_section(section)?,
    })
  }

  /// As `CEditShape::SaveLTX` writes it into one section.
  pub fn write_to_section(&self, section: &mut IniSection) {
    self.object.write_to_section(section);
    self.shape.write_to_section(section);
  }

  /// The elements with the object's scale applied, `CEditShape::ApplyScale`, which the build gives an entity holding
  /// the shape attached: a sphere's centre placed by the scale and its radius times the scale along x, a box's matrix
  /// scaled after itself (`mulA_43`). The object's position and turn are not applied.
  ///
  /// A unit scale leaves the elements as stored. Summed as `mul_43` sums them, a box's negative zero would turn
  /// positive, yet GSC's build kept it: vanilla `zaton`'s `zat_surge_hide_a2` and `zat_a2_sr_light` hold such zeros in
  /// `level.spawn`.
  pub fn get_scaled_elements(&self) -> Vec<EditableShapeElement> {
    if self.object.transform.scale == Vector3d::ONE {
      return self.shape.elements.clone();
    }

    let scale: Matrix4x4 = Matrix4x4::from_scale(&self.object.transform.scale);

    self
      .shape
      .elements
      .iter()
      .map(|element| match element {
        EditableShapeElement::Sphere { center, radius } => EditableShapeElement::Sphere {
          center: scale.transform_point(center),
          radius: radius * self.object.transform.scale.x,
        },
        EditableShapeElement::Box(matrix) => {
          EditableShapeElement::Box(scale.multiply_affine(&Matrix4x4::from(matrix)).to_affine())
        }
      })
      .collect()
  }
}

impl EditableToolObject for EditableShapeObject {
  /// `OBJCLASS_SHAPE`.
  const CLASS_ID: i32 = 4;
  const TOOL_NAME: &'static str = "shape";

  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    Self::read_from_section(object.section)
  }

  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    self.write_to_section(ini.section_mut(section));
  }
}
