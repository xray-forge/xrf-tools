use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableBone, EditableBoneMass, EditableDescription, EditableObjectFile, EditableTimeWidth};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector3d};
use xrf_ogf::{OgfFile, OgfModelType};
use xrf_skeleton::SkeletonJointLimit;

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::tests::fixtures::{UP, assemble, chain, corner, object, surface, with_links};

const SOURCE: &str = "props\\crate.object";

/// A quad on [`chain`], one corner blended between its first two bones, with a class script, LODs, motion
/// references, an authored description, and physics on its second bone.
pub(super) fn authored() -> XrfResult<EditableObjectFile> {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
  let mut bones: Vec<EditableBone> = chain();

  bones[1].joint.limits[0] = SkeletonJointLimit {
    min: -0.5,
    max: 0.25,
    spring_factor: 2.0,
    damping_factor: 3.0,
  };
  bones[1].friction = Some(0.3);
  bones[1].mass = Some(EditableBoneMass {
    mass: 2.0,
    center: Vector3d::new(0.0, 0.1, 0.0),
  });

  let mut authored: EditableObjectFile = object(
    assemble(&[surface(vec![
      [corner(a, UP), corner(b, UP), corner(c, UP)],
      [
        corner(a, UP),
        corner(c, UP),
        with_links(corner(d, UP), &[(0, 0.5), (1, 0.5)]),
      ],
    ])])?,
    bones,
  );

  authored.class_script = Some(String::from("[collide]\nignore_static"));
  authored.lods = Some(String::from("dynamics\\crate_lod"));
  authored.motion_references = Some(vec![String::from("crate_motions")]);
  authored.description = Some(EditableDescription {
    creator: String::from("maker"),
    created_at: 1_000,
    modifier: String::from("editor"),
    modified_at: 2_000,
    time_width: EditableTimeWidth::Seconds32,
  });

  Ok(authored)
}

pub(super) fn compile(object: &EditableObjectFile) -> XrfResult<OgfFile> {
  let source: ObjectCompileSource = ObjectCompileSource {
    source_file: String::from(SOURCE),
    convertor: String::from("xrf"),
    built_at: 3_000,
  };

  OgfFile::read_from_bytes::<XRayByteOrder>(ObjectCompiler::compile(object, &source)?)
}

#[test]
fn writes_an_ogf_the_reader_takes_whole() -> XrfResult {
  let file: OgfFile = compile(&authored()?)?;
  let child: &OgfFile = &file.children.as_ref().expect("Expect children").nested[0];

  assert!(file.residue.is_none(), "{:?}", file.residue);
  assert_eq!(file.header.model_type, OgfModelType::SkeletonAnim.to_raw());
  assert_eq!(
    file.header.bounding_box,
    Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::new(1.0, 0.0, 1.0)
    }
  );
  assert_eq!(file.children.as_ref().map(|it| it.nested.len()), Some(1));
  assert_eq!(child.header.model_type, OgfModelType::SkeletonGeomdefSt.to_raw());

  let geometry = child.geometry.as_ref().expect("Expect the child's geometry");

  assert_eq!(geometry.vertex_count, Some(6));
  assert_eq!(geometry.indices.as_ref().map(Vec::len), Some(6));

  // The authored `(-0.5, 0.25)` in the physics engine's sense.
  let stored: &SkeletonJointLimit = &file.ik_data.as_ref().expect("Expect IK data").bones[1].joint.limits[0];

  assert_eq!((stored.min, stored.max, stored.spring_factor), (-0.25, 0.5, 2.0));

  let description = file.get_description().expect("Expect a description");

  assert_eq!(
    (
      description.source_file.as_str(),
      description.built_at,
      description.created_at
    ),
    (SOURCE, 3_000, 1_000)
  );
  assert_eq!(
    file.kinematics.as_ref().map(|it| it.motion_refs.clone()),
    Some(vec![String::from("crate_motions")])
  );

  Ok(())
}

#[test]
fn boxes_each_bone_around_the_vertices_it_leads_in_its_rest_space() -> XrfResult {
  let file: OgfFile = compile(&authored()?)?;
  let bones = &file.bones.as_ref().expect("Expect bones").bones;

  // Every vertex leads with the root, the lowest bone it follows; the others lead none and keep the empty box.
  assert_eq!(bones[0].obb.center, Vector3d::new(0.5, 0.0, 0.5));
  assert_eq!(bones[0].obb.half_extents, Vector3d::new(0.5, 0.0, 0.5));
  assert_eq!(bones[1].obb.half_extents, Vector3d::ZERO);
  assert_eq!(
    bones.iter().map(|bone| bone.parent.as_str()).collect::<Vec<_>>(),
    ["", "link", "lid"]
  );

  Ok(())
}

#[test]
fn decompiles_back_to_the_authored_physics_script_and_lods() -> XrfResult {
  let authored: EditableObjectFile = authored()?;
  let decompiled: EditableObjectFile = ObjectDecompiler::decompile(&compile(&authored)?)?.object;
  let bone = |object: &EditableObjectFile| object.bones.as_ref().expect("Expect bones")[1].clone();
  let (before, after): (EditableBone, EditableBone) = (bone(&authored), bone(&decompiled));

  assert_eq!(after.joint.limits, before.joint.limits);
  assert_eq!(after.friction, before.friction);
  assert_eq!(after.mass, before.mass);
  assert_eq!(after.joint_flags, before.joint_flags);
  assert_eq!(decompiled.class_script, authored.class_script);
  assert_eq!(decompiled.lods, authored.lods);
  assert_eq!(decompiled.motion_references, authored.motion_references);
  assert_eq!(decompiled.description, authored.description);

  Ok(())
}
