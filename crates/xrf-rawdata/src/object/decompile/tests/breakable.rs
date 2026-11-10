use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableBone, EditableObjectFile};
use xrf_error::XrfResult;
use xrf_ogf::OgfFile;

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::object_compiler::ObjectCompiler;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::tests::fixtures::{UP, assemble, chain, corner, object, surface, with_links};

/// A quad cut into two fragments as the Level Editor cuts a breakable part: one triangle on each of the first two
/// bones, each bone of its own game material.
fn part() -> XrfResult<OgfFile> {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
  let on_second = |position: [f32; 3]| with_links(corner(position, UP), &[(1, 1.0)]);
  let mut bones: Vec<EditableBone> = chain();

  bones[0].game_material = String::from("materials\\glass");
  bones[1].game_material = String::from("materials\\wood");

  let authored: EditableObjectFile = object(
    assemble(&[surface(vec![
      [corner(a, UP), corner(b, UP), corner(c, UP)],
      [on_second(a), on_second(c), on_second(d)],
    ])])?,
    bones,
  );
  let source: ObjectCompileSource = ObjectCompileSource {
    source_file: String::from("brkbl#0"),
    convertor: String::from("xrf"),
    built_at: 0,
  };

  OgfFile::read_from_bytes::<XRayByteOrder>(ObjectCompiler::compile(&authored, &source)?.bytes)
}

/// The fragments' bones are dropped and their points welded back into one static mesh, a surface for each game
/// material the fragments carry.
#[test]
fn decompiles_a_breakable_part_into_the_static_faces_it_was_cut_from() -> XrfResult {
  let decompiled: EditableObjectFile = ObjectDecompiler::decompile_breakable(&part()?)?.object;
  let mesh = &decompiled.meshes.as_deref().unwrap_or_default()[0];
  let mut materials: Vec<&str> = decompiled
    .surfaces
    .as_deref()
    .unwrap_or_default()
    .iter()
    .map(|surface| surface.game_material.as_str())
    .collect();

  materials.sort_unstable();

  assert!(decompiled.bones.is_none(), "a breakable comes back without bones");
  assert_eq!(decompiled.flags, 0, "a breakable comes back static");
  assert_eq!(mesh.points.len(), 4, "the fragments' points weld where they met");
  assert_eq!(mesh.faces.len(), 2);
  assert_eq!(materials, ["materials\\glass", "materials\\wood"]);

  Ok(())
}

/// A part of no skeleton is no breakable part.
#[test]
fn refuses_a_part_that_is_no_rigid_skeleton() -> XrfResult {
  let mut file: OgfFile = part()?;

  file.header.model_type = 0;

  let error: String = ObjectDecompiler::decompile_breakable(&file)
    .expect_err("a part of no skeleton")
    .to_string();

  assert!(error.contains("rigid skeleton"), "{error}");

  Ok(())
}
