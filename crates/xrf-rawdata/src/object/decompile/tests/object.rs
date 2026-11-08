use xrf_chunk::{ChunkLayout, ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_editable_object::{EditableObjectFile, EditableSurface};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{
  OgfFile, OgfFvf, OgfGeometry, OgfHeaderChunk, OgfModelType, OgfSlideWindow, OgfStaticVertex, OgfSwiDataChunk,
  OgfTextureChunk,
};

use crate::object::decompile::geometry::source::ogf_corner::OgfCorner;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::tests::fixtures::{DOWN, UP, assemble, corner, surface, with_uv};

#[test]
fn cuts_an_ogf_string_at_the_terminator_it_kept() {
  assert_eq!(
    ObjectDecompiler::trim_text("[collide]\nignore_static\n\0"),
    "[collide]\nignore_static\n"
  );
  assert_eq!(ObjectDecompiler::trim_text("lods\\tree"), "lods\\tree");
  assert_eq!(ObjectDecompiler::trim_text(""), "");
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
    class_script: Some(ObjectDecompiler::trim_text("[collide]\n\0")),
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

/// A static visual of `model_type` holding one quad, `a b c` and `a c d` over four vertices, then a third face behind
/// the first sliding window where `windows` are given, as a progressive mesh keeps its coarser levels.
fn static_visual(model_type: OgfModelType, windows: Option<Vec<OgfSlideWindow>>) -> XrfResult<OgfFile> {
  let mut file: ChunkWriter = ChunkWriter::new();
  let mut chunk = |id: u32, write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult {
    let mut contents: ChunkWriter = ChunkWriter::new();

    write(&mut contents)?;
    contents.flush_chunk_into::<XRayByteOrder>(&mut file, id)?;

    Ok(())
  };
  let vertex = |position: [f32; 3]| OgfStaticVertex {
    position: Vector3d::from(position),
    normal: Some(Vector3d::Y),
    point_size: None,
    diffuse: None,
    specular: None,
    texture_coordinates: vec![vec![position[0], position[2]]],
  };

  chunk(OgfHeaderChunk::CHUNK_ID, &|writer| {
    OgfHeaderChunk {
      version: 4,
      model_type: model_type.to_raw(),
      shader_id: 0,
      bounding_box: Aabb {
        min: Vector3d::ZERO,
        max: Vector3d::new(1.0, 0.0, 1.0),
      },
      bounding_sphere: Sphere {
        center: Vector3d::ZERO,
        radius: 1.0,
      },
    }
    .write::<XRayByteOrder>(writer)
  })?;
  chunk(OgfTextureChunk::CHUNK_ID, &|writer| {
    OgfTextureChunk {
      texture_name: String::from("wood\\wood_plank6"),
      shader_name: String::from("default"),
      trailing: Vec::new(),
    }
    .write::<XRayByteOrder>(writer)
  })?;

  let mut geometry: ChunkWriter = ChunkWriter::new();

  OgfGeometry::write_static::<XRayByteOrder>(
    &mut geometry,
    &OgfFvf::from_raw(EditableSurface::POSITION_NORMAL_UV_FVF).expect("the export's format decodes"),
    &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]].map(vertex),
    &[0, 1, 2, 0, 2, 3, 0, 1, 3],
  )?;

  let mut bytes: Vec<u8> = file.flush_raw_into_buffer()?;

  bytes.extend(geometry.flush_raw_into_buffer()?);

  if let Some(windows) = windows {
    let mut swi: ChunkWriter = ChunkWriter::new();

    OgfSwiDataChunk {
      reserved: [0; 4],
      windows,
    }
    .write::<XRayByteOrder>(&mut swi)?;
    bytes.extend(swi.flush_chunk_into_buffer::<XRayByteOrder>(OgfSwiDataChunk::CHUNK_ID)?);
  }

  OgfFile::read_from_bytes::<XRayByteOrder>(bytes)
}

#[test]
fn decompiles_a_progressive_visual_from_its_full_detail_window() -> XrfResult {
  // The first window draws the quad's two faces from the start of the indices; the third face is a coarser level's.
  let file: OgfFile = static_visual(
    OgfModelType::Progressive,
    Some(vec![OgfSlideWindow {
      offset: 0,
      num_tris: 2,
      num_verts: 4,
    }]),
  )?;
  let object: EditableObjectFile = ObjectDecompiler::decompile(&file)?.object;

  assert_eq!(object.flags, EditableObjectFile::PROGRESSIVE_FLAG);
  assert_eq!(object.meshes.as_ref().map(|it| it[0].faces.len()), Some(2));

  // A visual of no sliding windows draws every face it stores.
  let file: OgfFile = static_visual(OgfModelType::Normal, None)?;
  let object: EditableObjectFile = ObjectDecompiler::decompile(&file)?.object;

  assert_eq!(object.flags, 0);
  assert_eq!(object.meshes.as_ref().map(|it| it[0].faces.len()), Some(3));

  Ok(())
}

#[test]
fn refuses_a_visual_no_editable_object_exports_to() -> XrfResult {
  let file: OgfFile = static_visual(OgfModelType::TreeSt, None)?;
  let error: String = ObjectDecompiler::decompile(&file)
    .expect_err("Expect a tree visual refused")
    .to_string();

  assert!(error.contains("MT_TREE_ST"), "{error}");

  Ok(())
}
