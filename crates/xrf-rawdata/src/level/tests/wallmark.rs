use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{EditableBinaryPart, EditableWallmark, EditableWallmarkSlot, EditableWallmarkTool};
use xrf_error::XrfResult;
use xrf_level::{LevelWallmark, LevelWallmarkSlot, LevelWallmarkVertex, LevelWallmarksFile};
use xrf_math::{Aabb, Vector3d};

use crate::LevelWallmarkConversion;
use crate::level::build::level_build_geometry::LevelBuildGeometry;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;
use crate::level::tests::fixtures::GUID;

fn decal(offset: f32, vertices: usize) -> LevelWallmark {
  let vertices: Vec<LevelWallmarkVertex> = (0..vertices)
    .map(|index| LevelWallmarkVertex {
      position: Vector3d::new(offset + index as f32 * 0.5, 1.25, -3.0 + (index % 2) as f32),
      color: 0xFF40_6080,
      u: index as f32 * 0.125,
      v: 0.5,
    })
    .collect();
  // A decal of no vertex, which only a broken file holds, keeps a sphere about the origin.
  let bounds: Aabb =
    Aabb::from_points(vertices.iter().map(|vertex| vertex.position)).unwrap_or_else(|| Aabb::of_point(&Vector3d::ZERO));

  LevelWallmark {
    center: bounds.get_center(),
    radius: bounds.get_bounding_sphere().radius,
    vertices,
  }
}

fn slot(texture: &str, marks: Vec<LevelWallmark>) -> LevelWallmarkSlot {
  LevelWallmarkSlot {
    shader: String::from("effects\\wallmarkmult"),
    texture: texture.to_owned(),
    marks,
  }
}

fn bytes_of(file: &LevelWallmarksFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

#[test]
fn wall_marks_come_back_to_the_same_file_through_the_part() -> XrfResult {
  let file: LevelWallmarksFile = LevelWallmarksFile {
    slots: vec![
      slot("decal\\decal_graz2", vec![decal(0.0, 4), decal(10.0, 7)]),
      slot("decal\\decal_blood", vec![decal(-5.5, 3)]),
    ],
  };
  let tool: EditableWallmarkTool = LevelWallmarkConversion::decompile(&file)?;
  let part: Vec<u8> = tool.to_part::<XRayByteOrder>(GUID)?.write_to_bytes::<XRayByteOrder>()?;
  let read: EditableWallmarkTool = EditableWallmarkTool::read_from_part::<XRayByteOrder>(
    &EditableBinaryPart::read_from_bytes::<XRayByteOrder>(part)?,
  )?;
  let mark: &EditableWallmark = &read.slots[0].marks[1];

  assert_eq!(read, tool);
  assert_eq!(mark.flags, 0, "no decal GSC saved is selected");
  assert_eq!((mark.width, mark.height, mark.rotation), (1.0, 1.0, 0.0), "the brush's");
  assert_eq!(mark.bbox, EditableWallmark::get_bounds(&mark.vertices).unwrap().0);
  assert_eq!(bytes_of(&LevelWallmarkConversion::export(&read))?, bytes_of(&file)?);

  Ok(())
}

/// A wall mark's triangle too small for xrLC goes into neither `build.prj` nor the face count, but stays in the wall
/// marks the game draws.
#[test]
fn leaves_a_wall_mark_triangle_of_no_area_out_of_the_build() -> XrfResult {
  let mut tiny: LevelWallmark = decal(0.0, 6);

  // The second triangle a millimetre across, as `jupiter`'s are.
  for (vertex, position) in tiny.vertices[3..].iter_mut().zip([
    Vector3d::new(5.0, 1.25, 0.0),
    Vector3d::new(5.001, 1.25, 0.0),
    Vector3d::new(5.0, 1.25, 0.0005),
  ]) {
    vertex.position = position;
  }

  let bounds: Aabb = Aabb::from_points(tiny.vertices.iter().map(|vertex| vertex.position)).expect("its vertices");

  tiny.center = bounds.get_center();
  tiny.radius = bounds.get_bounding_sphere().radius;

  let file: LevelWallmarksFile = LevelWallmarksFile {
    slots: vec![slot("decal\\decal_graz2", vec![tiny])],
  };
  let tool: EditableWallmarkTool = LevelWallmarkConversion::decompile(&file)?;
  let mut geometry: LevelBuildGeometry = LevelBuildGeometry::default();

  geometry.add_wallmarks(
    &tool,
    &LevelBuildSectors::new(&[], |_, _| None),
    &mut LevelBuildTables::default(),
  )?;

  assert_eq!((geometry.faces.len(), geometry.invalid_faces), (1, 1));
  assert_eq!(bytes_of(&LevelWallmarkConversion::export(&tool))?, bytes_of(&file)?);

  Ok(())
}

#[test]
fn exports_the_tool_as_the_editor_loaded_it() -> XrfResult {
  // `LoadStream` drops a fan past 512 vertices, and `RefiningSlots` the slot that leaves empty.
  let mut tool: EditableWallmarkTool = LevelWallmarkConversion::decompile(&LevelWallmarksFile {
    slots: vec![
      slot("decal\\decal_graz2", vec![decal(0.0, 4)]),
      slot(
        "decal\\decal_blood",
        vec![decal(0.0, EditableWallmark::MAX_VERTICES + 1)],
      ),
    ],
  })?;

  tool.slots.push(EditableWallmarkSlot {
    shader: String::new(),
    texture: String::new(),
    marks: Vec::new(),
  });

  let exported: LevelWallmarksFile = LevelWallmarkConversion::export(&tool);

  assert_eq!(exported.slots.len(), 1);
  assert_eq!(exported.slots[0].texture, "decal\\decal_graz2");

  Ok(())
}

#[test]
fn refuses_a_decal_of_no_vertex() {
  let file: LevelWallmarksFile = LevelWallmarksFile {
    slots: vec![slot("decal\\decal_graz2", vec![decal(0.0, 4), decal(0.0, 0)])],
  };
  let error: String = LevelWallmarkConversion::decompile(&file).unwrap_err().to_string();

  assert!(error.contains("Wall mark 1 of slot 0"), "{error}");
}
