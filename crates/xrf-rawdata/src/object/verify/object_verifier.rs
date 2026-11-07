use std::fmt::Debug;

use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_ogf::{OgfBoneIkData, OgfFile, OgfJointIkData, OgfModelType};

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::verify::object_verification::{ObjectGeometryCensus, ObjectVerification};
use crate::object::verify::ogf_material_geometry::{OgfMaterialComparison, OgfMaterialGeometry};

/// Compiles an object and compares the OGF with one it should stand in for, by what the engine reads: the model, the
/// skeleton and its physics, the motions, and the geometry.
///
/// Left out, as the compile builds them its own way and the engine draws either alike: the order of children, vertices
/// and triangles, tangents, bone boxes and the description's authoring stamps.
pub struct ObjectVerifier;

impl ObjectVerifier {
  /// Header bounds within this of the reference's are the same bounds.
  const BOUNDS_TOLERANCE: f32 = 1e-4;

  /// How `object` compiled compares with `reference`.
  ///
  /// # Errors
  ///
  /// Returns an error where the compile does, or where its OGF does not read back.
  pub fn verify(object: &EditableObjectFile, reference: &OgfFile) -> XrfResult<ObjectVerification> {
    let compiled: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(ObjectCompiler::compile(
      object,
      &ObjectCompileSource {
        source_file: String::new(),
        convertor: String::new(),
        built_at: 0,
      },
    )?)?;

    Ok(Self::compare(&compiled, reference))
  }

  /// How `compiled` compares with `reference`.
  fn compare(compiled: &OgfFile, reference: &OgfFile) -> ObjectVerification {
    let mut verification: ObjectVerification = ObjectVerification::default();

    Self::compare_model(compiled, reference, &mut verification.model);
    Self::compare_skeleton(compiled, reference, &mut verification.skeleton);
    Self::compare_motions(compiled, reference, &mut verification.motions);
    verification.census = Self::compare_geometry(compiled, reference, &mut verification.geometry);

    verification
  }

  fn compare_model(compiled: &OgfFile, reference: &OgfFile, differences: &mut Vec<String>) {
    let model_type = |file: &OgfFile| {
      OgfModelType::from_raw(file.header.model_type).map_or_else(
        || OgfModelType::label(file.header.model_type),
        |it| String::from(it.name()),
      )
    };

    Self::note(differences, "model type", model_type(compiled), model_type(reference));
    // The engine reads the user data as ini text, so a terminator one exporter wrote and another did not is no
    // difference.
    Self::note(
      differences,
      "user data",
      compiled
        .user_data
        .as_ref()
        .map(|it| it.user_data.trim_end_matches('\0')),
      reference
        .user_data
        .as_ref()
        .map(|it| it.user_data.trim_end_matches('\0')),
    );
    // The engine reads the LODs' first line alone, and exporters end it differently.
    Self::note(
      differences,
      "LODs",
      compiled.lods.as_ref().map(|it| it.get_visual_name()),
      reference.lods.as_ref().map(|it| it.get_visual_name()),
    );
    Self::note(
      differences,
      "motion references",
      compiled.kinematics.as_ref().map(|it| &it.motion_refs),
      reference.kinematics.as_ref().map(|it| &it.motion_refs),
    );
  }

  fn compare_skeleton(compiled: &OgfFile, reference: &OgfFile, differences: &mut Vec<String>) {
    let (names, reference_names) = (Self::bones(compiled), Self::bones(reference));

    if names.len() != reference_names.len() {
      differences.push(format!(
        "bones: {} compiled, {} in the reference",
        names.len(),
        reference_names.len()
      ));
    }

    for (index, (bone, other)) in names.iter().zip(&reference_names).enumerate() {
      if bone != other {
        differences.push(format!(
          "bone {index}: {} under '{}', the reference's {} under '{}'",
          bone.0, bone.1, other.0, other.1
        ));
      }
    }

    for ((bone, ik), other) in names.iter().zip(Self::physics(compiled)).zip(Self::physics(reference)) {
      let fields: [(&str, bool); 7] = [
        ("game material", ik.game_material == other.game_material),
        ("shape", ik.shape == other.shape),
        ("joint", Self::is_same_joint(&ik.joint, &other.joint)),
        ("mass", Self::is_same(ik.mass, other.mass)),
        (
          "centre of mass",
          Self::is_same_vector(&ik.center_of_mass, &other.center_of_mass),
        ),
        (
          "bind rotation",
          Self::is_same_vector(&ik.bind_rotation, &other.bind_rotation),
        ),
        (
          "bind position",
          Self::is_same_vector(&ik.bind_position, &other.bind_position),
        ),
      ];
      let differing: Vec<&str> = fields
        .iter()
        .filter(|(_, is_same)| !is_same)
        .map(|(field, _)| *field)
        .collect();

      if !differing.is_empty() {
        differences.push(format!("bone {}: {} differ", bone.0, differing.join(", ")));
      }
    }
  }

  /// Whether two stored values are one: equal, or both not a number. Some vanilla bones store a NaN friction, which a
  /// round trip keeps and `==` never finds equal to itself.
  fn is_same(first: f32, second: f32) -> bool {
    first == second || (first.is_nan() && second.is_nan())
  }

  fn is_same_vector(first: &Vector3d, second: &Vector3d) -> bool {
    first
      .to_array()
      .iter()
      .zip(second.to_array())
      .all(|(a, b)| Self::is_same(*a, b))
  }

  fn is_same_joint(first: &OgfJointIkData, second: &OgfJointIkData) -> bool {
    let factors = |joint: &OgfJointIkData| {
      [
        joint.spring_factor,
        joint.damping_factor,
        joint.break_force,
        joint.break_torque,
      ]
    };
    let limits = |joint: &OgfJointIkData| {
      joint
        .limits
        .each_ref()
        .map(|limit| [limit.min, limit.max, limit.spring_factor, limit.damping_factor])
    };

    first.kind == second.kind
      && first.ik_flags == second.ik_flags
      && factors(first)
        .iter()
        .zip(factors(second))
        .all(|(a, b)| Self::is_same(*a, b))
      && limits(first)
        .as_flattened()
        .iter()
        .zip(limits(second).as_flattened())
        .all(|(a, b)| Self::is_same(*a, *b))
      && match (first.friction, second.friction) {
        (Some(a), Some(b)) => Self::is_same(a, b),
        (a, b) => a.is_none() && b.is_none(),
      }
  }

  /// Each bone's name and its parent's, in the order stored.
  fn bones(file: &OgfFile) -> Vec<(&str, &str)> {
    file
      .bones
      .iter()
      .flat_map(|chunk| &chunk.bones)
      .map(|bone| (bone.name.as_str(), bone.parent.as_str()))
      .collect()
  }

  /// Each bone's physics, in the order of the bones.
  fn physics(file: &OgfFile) -> &[OgfBoneIkData] {
    file.ik_data.as_ref().map_or(&[], |it| it.bones.as_slice())
  }

  /// Each own motion's label and length, in the order stored.
  fn motions(file: &OgfFile) -> Vec<(&str, u32)> {
    file
      .motions
      .iter()
      .flat_map(|chunk| &chunk.motions)
      .map(|motion| (motion.label.as_str(), motion.count))
      .collect()
  }

  fn compare_motions(compiled: &OgfFile, reference: &OgfFile, differences: &mut Vec<String>) {
    let (own, reference_own) = (Self::motions(compiled), Self::motions(reference));

    if own.len() != reference_own.len() {
      differences.push(format!(
        "own motions: {} compiled, {} in the reference",
        own.len(),
        reference_own.len()
      ));
    }

    for ((label, count), (other_label, other_count)) in own.iter().zip(&reference_own) {
      if label != other_label || count != other_count {
        differences.push(format!(
          "motion {label} of {count} frames, the reference's {other_label} of {other_count}"
        ));
      }
    }

    Self::note(
      differences,
      "motion parts",
      compiled.motion_parameters.as_ref().map(|it| &it.parts),
      reference.motion_parameters.as_ref().map(|it| &it.parts),
    );
    Self::note(
      differences,
      "motion definitions",
      compiled.motion_parameters.as_ref().map(|it| &it.motions),
      reference.motion_parameters.as_ref().map(|it| &it.motions),
    );
  }

  fn compare_geometry(compiled: &OgfFile, reference: &OgfFile, differences: &mut Vec<String>) -> ObjectGeometryCensus {
    let is_near = |first: &Vector3d, second: &Vector3d| {
      first
        .to_array()
        .iter()
        .zip(second.to_array())
        .all(|(a, b)| (a - b).abs() <= Self::BOUNDS_TOLERANCE)
    };
    let (bounds, reference_bounds) = (&compiled.header.bounding_box, &reference.header.bounding_box);

    if !is_near(&bounds.min, &reference_bounds.min) || !is_near(&bounds.max, &reference_bounds.max) {
      differences.push(format!("bounds {bounds:?}, the reference's {reference_bounds:?}"));
    }

    // The export orders children by where their surfaces sat in the editor's memory, which nothing records, and where a
    // surface's faces fall among children is the exporter's choice, so each texture and shader is compared whole.
    let materials: Vec<OgfMaterialGeometry> = OgfMaterialGeometry::collect(compiled);
    let reference_materials: Vec<OgfMaterialGeometry> = OgfMaterialGeometry::collect(reference);
    let mut census: ObjectGeometryCensus = ObjectGeometryCensus {
      children: materials.iter().map(|it| it.children).sum(),
      reference_children: reference_materials.iter().map(|it| it.children).sum(),
      ..ObjectGeometryCensus::default()
    };

    for material in &materials {
      census.vertices += material.vertices.len();
      census.faces += material.faces.len();

      match reference_materials
        .iter()
        .find(|other| other.texture == material.texture && other.shader == material.shader)
      {
        Some(other) => Self::compare_material(material, other, &mut census, differences),
        None => differences.push(format!(
          "{} / {} is not drawn by the reference",
          material.texture, material.shader
        )),
      }
    }

    for other in reference_materials.iter().filter(|other| {
      !materials
        .iter()
        .any(|it| it.texture == other.texture && it.shader == other.shader)
    }) {
      census.reference_faces += other.faces.len();
      differences.push(format!(
        "the reference's {} / {} is not compiled",
        other.texture, other.shader
      ));
    }

    census
  }

  fn compare_material(
    material: &OgfMaterialGeometry,
    other: &OgfMaterialGeometry,
    census: &mut ObjectGeometryCensus,
    differences: &mut Vec<String>,
  ) {
    let comparison: OgfMaterialComparison = material.compare(other);

    census.vertices_matched += comparison.vertices_matched;
    census.vertices_normal_only += comparison.vertices_normal_only;
    census.reference_faces += other.faces.len();
    census.faces_matched += comparison.faces_matched;
    census.reference_flat_faces += comparison.reference_flat_faces;

    if comparison.vertices_matched != material.vertices.len() {
      differences.push(format!(
        "{} / {}: {} of {} vertices have no reference vertex like them, {} of them differing in their normal alone",
        material.texture,
        material.shader,
        material.vertices.len() - comparison.vertices_matched,
        material.vertices.len(),
        comparison.vertices_normal_only
      ));
    }

    if comparison.faces_missing > 0 || comparison.faces_extra > 0 {
      differences.push(format!(
        "{} / {}: {} reference faces not drawn, {} drawn the reference does not",
        material.texture, material.shader, comparison.faces_missing, comparison.faces_extra
      ));
    }
  }

  /// Notes `label` as differing where `compiled` and `reference` are not equal.
  fn note<T: PartialEq + Debug>(differences: &mut Vec<String>, label: &str, compiled: T, reference: T) {
    if compiled != reference {
      differences.push(format!("{label}: {compiled:?}, the reference's {reference:?}"));
    }
  }
}
