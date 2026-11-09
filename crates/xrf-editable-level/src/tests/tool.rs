use byteorder::WriteBytesExt;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};

use crate::tests::fixtures::GUID;
use crate::{
  EditableBinaryPart, EditableLevelPart, EditableWallmark, EditableWallmarkSlot, EditableWallmarkTool,
  EditableWallmarkVertex,
};

fn fan() -> Vec<EditableWallmarkVertex> {
  [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 1.0, 0.5], [0.0, 1.0, 0.5]]
    .into_iter()
    .enumerate()
    .map(|(index, position)| EditableWallmarkVertex {
      position: Vector3d::from(position),
      color: 0xFF80_8080,
      u: index as f32 * 0.25,
      v: 1.0,
    })
    .collect()
}

fn mark(width: f32) -> EditableWallmark {
  let vertices: Vec<EditableWallmarkVertex> = fan();
  let (bbox, bounds) = EditableWallmark::get_bounds(&vertices).unwrap();

  EditableWallmark {
    flags: 0,
    bbox,
    bounds,
    width,
    height: width,
    rotation: 0.0,
    vertices,
  }
}

fn tool() -> EditableWallmarkTool {
  EditableWallmarkTool {
    texture: String::from("decal\\decal_graz2"),
    ..EditableWallmarkTool::new(vec![
      EditableWallmarkSlot {
        shader: String::from("effects\\wallmarkmult"),
        texture: String::from("decal\\decal_graz2"),
        marks: vec![mark(2.5), mark(1.5)],
      },
      EditableWallmarkSlot {
        shader: String::new(),
        texture: String::new(),
        marks: Vec::new(),
      },
    ])
  }
}

#[test]
fn a_wall_mark_tool_saved_as_its_part_reads_back_the_same() -> XrfResult {
  let bytes: Vec<u8> = tool()
    .to_part::<XRayByteOrder>(GUID)?
    .write_to_bytes::<XRayByteOrder>()?;
  let EditableLevelPart::Binary(part) = EditableLevelPart::read_from_bytes::<XRayByteOrder>(bytes.clone())? else {
    panic!("a wall mark part is chunks");
  };
  let read: EditableWallmarkTool = EditableWallmarkTool::read_from_part::<XRayByteOrder>(&part)?;

  assert_eq!(part.guid, GUID);
  assert_eq!(read, tool(), "an empty slot kept as stored");
  assert_eq!(
    read.to_part::<XRayByteOrder>(GUID)?.write_to_bytes::<XRayByteOrder>()?,
    bytes
  );

  Ok(())
}

#[test]
fn reads_a_version_3_stream_as_the_editor_does() -> XrfResult {
  // Version 3 keeps decals in chunk 4 without their size and turn, and its brush shader is replaced on load.
  let mut stream: ChunkWriter = ChunkWriter::new();
  let mut chunk = |id: u32, write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;
    writer.flush_chunk_into::<XRayByteOrder>(&mut stream.buffer, id)?;

    Ok(())
  };

  chunk(1, &|writer| Ok(writer.write_u16::<XRayByteOrder>(3)?))?;
  chunk(2, &|writer| Ok(writer.write_u32::<XRayByteOrder>(1)?))?;
  chunk(3, &|writer| {
    for value in [1.0, 1.0, 0.0] {
      writer.write_f32::<XRayByteOrder>(value)?;
    }

    writer.write_w1251_string("effects\\wallmarkblend")?;
    writer.write_w1251_string("decal\\decal_graz2")?;

    Ok(())
  })?;
  chunk(4, &|writer| {
    let mut slot: ChunkWriter = ChunkWriter::new();
    let mark: EditableWallmark = mark(1.0);

    slot.write_u32::<XRayByteOrder>(1)?;
    slot.write_w1251_string("effects\\wallmarkmult")?;
    slot.write_w1251_string("decal\\decal_graz2")?;
    slot.write_u8(mark.flags)?;
    slot.write_xr::<XRayByteOrder, _>(&mark.bbox)?;
    slot.write_xr::<XRayByteOrder, _>(&mark.bounds)?;
    slot.write_u32::<XRayByteOrder>(mark.vertices.len() as u32)?;

    for vertex in &mark.vertices {
      vertex.write::<XRayByteOrder>(&mut slot)?;
    }

    slot.flush_chunk_into::<XRayByteOrder>(&mut writer.buffer, 0)?;

    Ok(())
  })?;

  let read: EditableWallmarkTool = EditableWallmarkTool::read_from_bytes::<XRayByteOrder>(stream.buffer)?;

  assert_eq!(read.shader, EditableWallmarkTool::DEFAULT_SHADER);
  assert_eq!(
    read.slots[0].marks,
    [mark(1.0)].map(|mark| EditableWallmark { rotation: 1.0, ..mark })
  );

  Ok(())
}

#[test]
fn refuses_a_stream_the_editor_does_not_load() -> XrfResult {
  let mut stream: ChunkWriter = ChunkWriter::new();
  let mut version: ChunkWriter = ChunkWriter::new();

  version.write_u16::<XRayByteOrder>(5)?;
  version.flush_chunk_into::<XRayByteOrder>(&mut stream.buffer, 1)?;

  let foreign: EditableBinaryPart = EditableBinaryPart {
    class_id: 13,
    ..tool().to_part::<XRayByteOrder>(GUID)?
  };

  assert!(
    EditableWallmarkTool::read_from_bytes::<XRayByteOrder>(stream.buffer).is_err(),
    "version 5"
  );
  assert!(
    EditableWallmarkTool::read_from_part::<XRayByteOrder>(&foreign).is_err(),
    "the AI map's part"
  );

  Ok(())
}

#[test]
fn bounds_a_fan_as_the_editor_places_a_decal() {
  let (bbox, bounds) = EditableWallmark::get_bounds(&fan()).unwrap();
  let fan_box: Aabb = Aabb {
    min: Vector3d::ZERO,
    max: Vector3d::new(2.0, 1.0, 0.5),
  };

  assert_eq!(bbox.min, Vector3d::new(-0.001, -0.001, -0.001));
  assert_eq!(bbox.max, Vector3d::new(2.0 + 0.001, 1.0 + 0.001, 0.5 + 0.001));
  assert_eq!(
    bounds,
    fan_box.get_bounding_sphere(),
    "the sphere about the box before it grows"
  );
  assert_eq!(EditableWallmark::get_bounds(&[]), None::<(Aabb, Sphere)>);
}
