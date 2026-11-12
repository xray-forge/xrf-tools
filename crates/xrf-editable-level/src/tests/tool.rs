use byteorder::WriteBytesExt;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};

use crate::tests::fixtures::GUID;
use crate::{
  EditableAiMapNode, EditableAiMapParams, EditableAiMapTool, EditableBinaryPart, EditableDetailColorIndex,
  EditableDetailHeader, EditableDetailObject, EditableDetailObjectTool, EditableLevelPart, EditableSceneTool,
  EditableWallmark, EditableWallmarkSlot, EditableWallmarkTool, EditableWallmarkVertex,
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
fn a_wall_mark_tool_becomes_the_scene_tool_of_its_part() -> XrfResult {
  let scene_tool: EditableSceneTool = tool().to_scene_tool::<XRayByteOrder>(GUID)?;

  assert_eq!(scene_tool.get_file_name(0), "wallmark.part");
  assert_eq!(
    scene_tool.parts,
    [EditableLevelPart::Binary(tool().to_part::<XRayByteOrder>(GUID)?)]
  );
  assert_eq!(
    EditableWallmarkTool::read_from_scene_tool::<XRayByteOrder>(&scene_tool)?,
    tool()
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

fn detail_tool() -> EditableDetailObjectTool {
  EditableDetailObjectTool {
    flags: EditableDetailObjectTool::OBJECTS_DRAW,
    header: EditableDetailHeader {
      version: 3,
      object_count: 2,
      offset_x: -41,
      offset_z: 216,
      size_x: 2,
      size_z: 1,
    },
    objects: vec![
      EditableDetailObject::new("detail\\trava_green", 1.5, 2.0, 0),
      EditableDetailObject {
        density_factor: 0.5,
        ..EditableDetailObject::new("detail\\det_list_01", 0.8, 1.3, EditableDetailObject::NO_WAVING_FLAG)
      },
    ],
    color_indices: vec![EditableDetailColorIndex {
      color: 0x00FF_8000,
      objects: vec![String::from("detail\\trava_green")],
    }],
    slots: vec![[0x0410_03E8, 0xFFFF_F1FF, 0x1111, 0x2222], [0x1234, 0, 0, 0]],
    bbox: Aabb {
      min: Vector3d::new(82.0, -3.5, -432.0),
      max: Vector3d::new(86.0, 12.0, -430.0),
    },
    base_texture: Some(String::from("terrain\\terrain_mp_darkvalley_det")),
    density: 0.3,
    snap_objects: vec![String::from("mp_darkvalley_sector_0")],
  }
}

#[test]
fn a_detail_object_tool_saved_as_its_part_reads_back_the_same() -> XrfResult {
  let scene_tool: EditableSceneTool = detail_tool().to_scene_tool::<XRayByteOrder>(GUID)?;
  let EditableLevelPart::Binary(part) = &scene_tool.parts[0] else {
    panic!("a detail part is chunks");
  };
  let bytes: Vec<u8> = part.write_to_bytes::<XRayByteOrder>()?;
  let EditableLevelPart::Binary(read) = EditableLevelPart::read_from_bytes::<XRayByteOrder>(bytes.clone())? else {
    panic!("a detail part reads as chunks");
  };

  assert_eq!(scene_tool.get_file_name(0), "detail_object.part");
  assert_eq!(
    EditableDetailObjectTool::read_from_part::<XRayByteOrder>(&read)?,
    detail_tool()
  );
  assert_eq!(read.write_to_bytes::<XRayByteOrder>()?, bytes);
  assert!(
    EditableDetailObjectTool::read_from_scene_tool::<XRayByteOrder>(&tool().to_scene_tool::<XRayByteOrder>(GUID)?)
      .is_err(),
    "the wall mark tool's part"
  );

  Ok(())
}

fn ai_map_tool() -> EditableAiMapTool {
  EditableAiMapTool {
    flags: 4,
    snap_objects: vec![String::from("labx8_sector_0")],
    ..EditableAiMapTool::new(
      Aabb {
        min: Vector3d::new(-10.0, -2.0, 5.0),
        max: Vector3d::new(10.0, 8.0, 25.0),
      },
      EditableAiMapParams::default(),
      vec![
        EditableAiMapNode {
          links: [1, EditableAiMapNode::NO_LINK, 0x00AB_CDEF, 0],
          plane: 0x8001,
          x: -14,
          z: 30,
          y: 13107,
          flags: 1,
        },
        EditableAiMapNode {
          links: [EditableAiMapNode::NO_LINK; 4],
          plane: 7,
          x: 3,
          z: 7,
          y: u16::MAX,
          flags: 0,
        },
      ],
    )
  }
}

#[test]
fn an_ai_map_tool_saved_as_its_part_reads_back_the_same() -> XrfResult {
  let scene_tool: EditableSceneTool = ai_map_tool().to_scene_tool::<XRayByteOrder>(GUID)?;
  let EditableLevelPart::Binary(part) = &scene_tool.parts[0] else {
    panic!("an AI map part is chunks");
  };
  let bytes: Vec<u8> = part.write_to_bytes::<XRayByteOrder>()?;
  let EditableLevelPart::Binary(read) = EditableLevelPart::read_from_bytes::<XRayByteOrder>(bytes.clone())? else {
    panic!("an AI map part reads as chunks");
  };

  assert_eq!(scene_tool.get_file_name(0), "ai_map.part");
  assert_eq!(
    EditableAiMapTool::read_from_part::<XRayByteOrder>(&read)?,
    ai_map_tool()
  );
  assert_eq!(read.write_to_bytes::<XRayByteOrder>()?, bytes);
  assert!(
    EditableAiMapTool::read_from_part::<XRayByteOrder>(&EditableBinaryPart {
      class_id: EditableDetailObjectTool::CLASS_ID,
      ..read
    })
    .is_err(),
    "the detail tool's part"
  );

  Ok(())
}

/// A node stands on its cell, the patch size from the world origin, at its height spread across the box.
#[test]
fn unpacks_a_nodes_position_as_the_editor_does() {
  let tool: EditableAiMapTool = ai_map_tool();
  let position: Vector3d = tool.get_node_position(&tool.nodes[0]);

  assert!((position.x + 9.8).abs() < 1e-5, "{position:?}");
  assert!(
    (position.y - 0.0).abs() < 1e-5,
    "a fifth of the way up a box ten high from -2"
  );
  assert!((position.z - 21.0).abs() < 1e-5);
  assert_eq!(tool.get_node_position(&tool.nodes[1]).y, 8.0);

  let mut wide: EditableAiMapTool = ai_map_tool();

  wide.nodes[0].links[0] = EditableAiMapNode::NO_LINK + 1;

  assert!(
    wide.write_to_bytes::<XRayByteOrder>().is_err(),
    "a link past three bytes"
  );
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
