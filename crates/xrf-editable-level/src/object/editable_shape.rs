use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::{Aabb, Affine3d, Matrix4x4, Vector3d};

use crate::level::editable_level_value::EditableLevelValue;

/// One element of a shape, `CShapeData::shape_def`, in the object's own space.
#[derive(Clone, Debug, PartialEq)]
pub enum EditableShapeElement {
  Sphere {
    center: Vector3d,
    radius: f32,
  },
  /// The matrix the unit box (`-0.5` to `0.5` on each axis) is placed by.
  Box(Affine3d),
}

/// The volume a shape object encloses, `CEditShape`: spheres and boxes, saved after the object's own fields
/// (`CEditShape::SaveLTX`). Sound environments, restrictors and level bounds are shapes.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableShape {
  /// `shape_type`, what the shape is used for (`eShapeUsage`): 0 for common, 1 for the level's bound.
  pub usage: u8,
  pub elements: Vec<EditableShapeElement>,
}

impl EditableShape {
  /// `SHAPE_CURRENT_VERSION`, which a shape object writes as its `version` unless its own class writes another over it.
  pub const VERSION: u32 = 2;

  /// `eShapeLevelBound`, the usage of the shape whose box the build takes for the level's map.
  pub const LEVEL_BOUND_USAGE: u8 = 1;

  /// `cfSphere` and `cfBox`, an element's `shape_type_N`.
  pub const SPHERE: u8 = 0;
  pub const BOX: u8 = 1;

  /// `du_box_vertices`, the corners of the unit box a box element's matrix places, in the editor's order.
  pub const UNIT_BOX_CORNERS: [[f32; 3]; 8] = [
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, -0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [0.5, -0.5, 0.5],
  ];

  /// The unit box's corners placed by `matrix`, each by `Fmatrix::transform_tiny`, in [`Self::UNIT_BOX_CORNERS`]'
  /// order.
  pub fn place_unit_box(matrix: &Matrix4x4) -> [Vector3d; 8] {
    Self::UNIT_BOX_CORNERS.map(|corner| matrix.transform_point(&Vector3d::from(corner)))
  }

  /// The box holding every element in the object's own space, `CEditShape::ComputeBounds`: a sphere's lowest and
  /// highest corner, a box's eight placed corners. None for a shape of no elements, whose bounds the editor leaves
  /// invalid.
  pub fn get_bounds(&self) -> Option<Aabb> {
    Aabb::from_points(self.elements.iter().flat_map(|element| match element {
      EditableShapeElement::Sphere { center, radius } => vec![
        Vector3d::new(center.x - radius, center.y - radius, center.z - radius),
        Vector3d::new(center.x + radius, center.y + radius, center.z + radius),
      ],
      EditableShapeElement::Box(matrix) => Self::place_unit_box(&Matrix4x4::from(matrix)).to_vec(),
    }))
  }

  /// The shape a new sound environment or shape takes: one unit box (`add_box(Fidentity)`).
  pub fn new_unit_box() -> Self {
    Self {
      usage: 0,
      elements: vec![EditableShapeElement::Box(Affine3d::IDENTITY)],
    }
  }

  /// As `CEditShape::LoadLTX` reads it: `shape_type` only from version 2, whichever class wrote the `version` it sees.
  ///
  /// # Errors
  ///
  /// Returns an error naming the field that is missing or malformed, or an element of an unknown type.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    let version: u32 = EditableLevelValue::read_u32(section, "version")?;
    let count: u32 = EditableLevelValue::read_u32(section, "shapes_count")?;
    let elements: Vec<EditableShapeElement> = (0..count)
      .map(
        |index| match section.get_parsed::<u8>(&format!("shape_type_{index}"))? {
          Self::SPHERE => Ok(EditableShapeElement::Sphere {
            center: EditableLevelValue::read_vector3(section, &format!("shape_center_{index}"))?,
            radius: section.get_parsed::<f32>(&format!("shape_radius_{index}"))?,
          }),
          Self::BOX => Ok(EditableShapeElement::Box(Affine3d {
            x_axis: EditableLevelValue::read_vector3(section, &format!("shape_matrix_i_{index}"))?,
            y_axis: EditableLevelValue::read_vector3(section, &format!("shape_matrix_j_{index}"))?,
            z_axis: EditableLevelValue::read_vector3(section, &format!("shape_matrix_k_{index}"))?,
            translation: EditableLevelValue::read_vector3(section, &format!("shape_matrix_c_{index}"))?,
          })),
          kind => Err(XrfError::new_parsing_error(format!(
            "Shape [{}] element {index} is of type {kind}, neither a sphere nor a box",
            section.name
          ))),
        },
      )
      .collect::<XrfResult<Vec<EditableShapeElement>>>()?;

    Ok(Self {
      usage: if version > 1 {
        section.get_parsed::<u8>("shape_type")?
      } else {
        0
      },
      elements,
    })
  }

  /// As `CEditShape::SaveLTX` writes it, at [`Self::VERSION`].
  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("version", Self::VERSION.to_string());
    section.set("shapes_count", self.elements.len().to_string());
    section.set("shape_type", self.usage.to_string());

    for (index, element) in self.elements.iter().enumerate() {
      match element {
        EditableShapeElement::Sphere { center, radius } => {
          section.set(format!("shape_type_{index}"), Self::SPHERE.to_string());
          section.set(
            format!("shape_center_{index}"),
            EditableLevelValue::format_vector3(center),
          );
          section.set(format!("shape_radius_{index}"), EditableLevelValue::format_f32(*radius));
        }
        EditableShapeElement::Box(matrix) => {
          section.set(format!("shape_type_{index}"), Self::BOX.to_string());

          for (axis, value) in [
            ("i", &matrix.x_axis),
            ("j", &matrix.y_axis),
            ("k", &matrix.z_axis),
            ("c", &matrix.translation),
          ] {
            section.set(
              format!("shape_matrix_{axis}_{index}"),
              EditableLevelValue::format_vector3(value),
            );
          }
        }
      }
    }
  }
}
