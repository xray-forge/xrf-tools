use xrf_chunk::{ChunkLayout, XRayByteOrder};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;

use crate::object::decompile::geometry::ogf_corner::OgfCorner;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::tests::fixtures::{DOWN, UP, assemble, corner, surface, with_uv};

#[test]
fn cuts_an_ogf_string_at_the_terminator_it_kept() {
  assert_eq!(
    ObjectDecompiler::to_text("[collide]\nignore_static\n\0"),
    "[collide]\nignore_static\n"
  );
  assert_eq!(ObjectDecompiler::to_text("lods\\tree"), "lods\\tree");
  assert_eq!(ObjectDecompiler::to_text(""), "");
}

#[test]
fn writes_an_object_that_reads_back_and_rewrites_alike() -> XrfResult {
  let [a, b, c, d] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
  // A UV seam, a plain face and a two-sided one: every vertex map and both surface kinds.
  let triangles: Vec<[OgfCorner; 3]> = vec![
    [corner(a, UP), corner(b, UP), corner(c, UP)],
    [with_uv(corner(a, UP), [0.5, 0.5]), corner(c, UP), corner(d, UP)],
    [corner(a, DOWN), corner(d, DOWN), corner(c, DOWN)],
  ];
  let assembled: AssembledMesh = assemble(&[surface(triangles)])?;
  let object: EditableObjectFile = EditableObjectFile {
    flags: EditableObjectFile::DYNAMIC_FLAG | EditableObjectFile::PROGRESSIVE_FLAG,
    flags_size: EditableObjectFile::FLAGS_SIZE,
    class_script: Some(ObjectDecompiler::to_text("[collide]\n\0")),
    lods: None,
    meshes: Some(vec![assembled.mesh]),
    surfaces: Some(assembled.surfaces),
    bones: None,
    embedded_motions: None,
    motion_references: None,
    bone_parts: None,
    actor_transform: None,
    description: None,
    layout: ChunkLayout::default(),
  };

  let written: Vec<u8> = object.write_to_bytes::<XRayByteOrder>()?;
  let read: EditableObjectFile = EditableObjectFile::read_from_bytes::<XRayByteOrder>(written.clone())?;

  assert_eq!(read.class_script.as_deref(), Some("[collide]\n"));
  assert_eq!(read.write_to_bytes::<XRayByteOrder>()?, written);

  Ok(())
}
