use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkLayout, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};
use xrf_utils::narrow_format_size;

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_binary_part::EditableBinaryPart;
use crate::part::editable_level_part::EditableLevelPart;
use crate::scene::editable_scene_tool::EditableSceneTool;
use crate::tool::editable_ai_map_node::EditableAiMapNode;

/// The settings the AI map was generated with, `SAIParams` (`ESceneAIMapTools_Export.h`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableAiMapParams {
  /// The side of one node's square cell, in metres.
  pub patch_size: f32,
  /// The height of the tester the editor walks over the scene, and how far up and down a neighbour may lie.
  pub test_height: f32,
  pub can_up: f32,
  pub can_down: f32,
}

impl Default for EditableAiMapParams {
  /// `SAIParams`' defaults.
  fn default() -> Self {
    Self {
      patch_size: 0.7,
      test_height: 1.0,
      can_up: 1.5,
      can_down: 4.0,
    }
  }
}

/// The AI map tool's part, `ai_map.part`: its nodes and the settings they were generated with, as
/// `ESceneAIMapTool::SaveStream` (`LevelEditor/Edit/ESceneAIMapTools.cpp:393`) writes them into the part's data chunk.
/// The Build exports the nodes into `build.aimap`, which xrAI turns into `level.ai`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableAiMapTool {
  /// The tool's flags, `m_Flags`.
  pub flags: u32,
  /// The box the nodes' heights are packed across.
  pub bounds: Aabb,
  pub params: EditableAiMapParams,
  pub nodes: Vec<EditableAiMapNode>,
  /// The editor's own settings: how far nodes are drawn, the brush, and the smoothing height.
  pub vis_radius: f32,
  pub brush_size: u32,
  pub smooth_height: f32,
  /// The scene objects the nodes were generated over, by name.
  pub snap_objects: Vec<String>,
}

impl EditableAiMapTool {
  /// `OBJCLASS_AIMAP`, which names the part's data chunk.
  pub const CLASS_ID: u32 = 13;

  /// The tool's `ClassName`, which names its part file.
  pub const TOOL_NAME: &'static str = "ai_map";

  /// `AIMAP_VERSION`, the only one the editor loads.
  pub const VERSION: u16 = 2;

  /// What a new tool takes: no flags, nodes drawn 30 metres out, a brush of one, a smoothing height of 0.5.
  pub const DEFAULT_VIS_RADIUS: f32 = 30.0;
  pub const DEFAULT_BRUSH_SIZE: u32 = 1;
  pub const DEFAULT_SMOOTH_HEIGHT: f32 = 0.5;

  const VERSION_CHUNK_ID: u32 = 0x0001;
  const FLAGS_CHUNK_ID: u32 = 0x0002;
  const BOX_CHUNK_ID: u32 = 0x0003;
  const PARAMS_CHUNK_ID: u32 = 0x0004;
  const NODES_CHUNK_ID: u32 = 0x0006;
  const SNAP_OBJECTS_CHUNK_ID: u32 = 0x0007;
  const INTERNAL_DATA_CHUNK_ID: u32 = 0x0008;
  const INTERNAL_DATA2_CHUNK_ID: u32 = 0x0009;

  /// A tool of `nodes` packed against `bounds` with `params`, the editor's own settings a new tool's.
  pub fn new(bounds: Aabb, params: EditableAiMapParams, nodes: Vec<EditableAiMapNode>) -> Self {
    Self {
      flags: 0,
      bounds,
      params,
      nodes,
      vis_radius: Self::DEFAULT_VIS_RADIUS,
      brush_size: Self::DEFAULT_BRUSH_SIZE,
      smooth_height: Self::DEFAULT_SMOOTH_HEIGHT,
      snap_objects: Vec::new(),
    }
  }

  /// Where `node` stands, `ESceneAIMapTool::UnpackPosition`: its cell scaled by the patch size, its height spread
  /// across the box.
  pub fn get_node_position(&self, node: &EditableAiMapNode) -> Vector3d {
    node.position.unpack(self.params.patch_size, &self.bounds)
  }

  /// # Errors
  ///
  /// Returns an error when the part is another tool's, or its data is not a stream this reads.
  pub fn read_from_part<T: ByteOrder>(part: &EditableBinaryPart) -> XrfResult<Self> {
    if part.class_id != Self::CLASS_ID {
      return Err(XrfError::new_parsing_error(format!(
        "Level part of class {} is not the AI map tool's {}",
        part.class_id,
        Self::CLASS_ID
      )));
    }

    Self::read_from_bytes::<T>(part.data.clone())
  }

  /// The tool's stream as `ESceneAIMapTool::LoadStream` reads it.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the editor asserts on is missing, the version is another, or a chunk does not hold
  /// exactly what it counts.
  pub fn read_from_bytes<T: ByteOrder>(data: Vec<u8>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(data)?.read_children()?;
    let version: u16 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u16::<T>()?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "AI map tool version {version} is not {}",
        Self::VERSION
      )));
    }

    let flags: u32 = ChunkReader::find_required_by_id(&chunks, Self::FLAGS_CHUNK_ID)?.read_u32::<T>()?;
    let mut bounds: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::BOX_CHUNK_ID)?;
    let bounds: Aabb = Aabb {
      min: Self::read_vector::<T>(&mut bounds)?,
      max: Self::read_vector::<T>(&mut bounds)?,
    };
    let mut params: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::PARAMS_CHUNK_ID)?;
    let params: EditableAiMapParams = EditableAiMapParams {
      patch_size: params.read_f32::<T>()?,
      test_height: params.read_f32::<T>()?,
      can_up: params.read_f32::<T>()?,
      can_down: params.read_f32::<T>()?,
    };
    let mut nodes_chunk: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::NODES_CHUNK_ID)?;
    let count: u32 = nodes_chunk.read_u32::<T>()?;
    let mut nodes: Vec<EditableAiMapNode> =
      nodes_chunk.new_bounded_vec(count.into(), EditableAiMapNode::SERIALIZED_SIZE, "AI map nodes")?;

    for _ in 0..count {
      nodes.push(EditableAiMapNode::read::<T>(&mut nodes_chunk)?);
    }

    nodes_chunk.assert_read("Expect the AI map nodes chunk to hold exactly the nodes it counts")?;

    let mut tool: Self = Self {
      flags,
      ..Self::new(bounds, params, nodes)
    };

    for mut chunk in chunks {
      match chunk.id {
        Self::INTERNAL_DATA_CHUNK_ID => {
          tool.vis_radius = chunk.read_f32::<T>()?;
          tool.brush_size = chunk.read_u32::<T>()?;
        }
        Self::INTERNAL_DATA2_CHUNK_ID => tool.smooth_height = chunk.read_f32::<T>()?,
        Self::SNAP_OBJECTS_CHUNK_ID => {
          let count: u32 = chunk.read_u32::<T>()?;

          tool.snap_objects = (0..count)
            .map(|_| chunk.read_w1251_string())
            .collect::<XrfResult<_>>()?;
        }
        _ => {}
      }
    }

    Ok(tool)
  }

  /// The tool's stream as `ESceneAIMapTool::SaveStream` writes it, at [`Self::VERSION`].
  ///
  /// # Errors
  ///
  /// Returns an error when a count is past what its `u32` holds, a link past its three bytes, or a name does not
  /// encode.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut stream: ChunkWriter = ChunkWriter::new();
    let mut chunk = |id: u32, write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult {
      let mut writer: ChunkWriter = ChunkWriter::new();

      write(&mut writer)?;
      writer.flush_chunk_into::<T>(&mut stream.buffer, id)?;

      Ok(())
    };

    chunk(Self::VERSION_CHUNK_ID, &|writer| {
      Ok(writer.write_u16::<T>(Self::VERSION)?)
    })?;
    chunk(Self::FLAGS_CHUNK_ID, &|writer| Ok(writer.write_u32::<T>(self.flags)?))?;
    chunk(Self::BOX_CHUNK_ID, &|writer| {
      for value in [self.bounds.min, self.bounds.max]
        .iter()
        .flat_map(|corner| [corner.x, corner.y, corner.z])
      {
        writer.write_f32::<T>(value)?;
      }

      Ok(())
    })?;
    chunk(Self::PARAMS_CHUNK_ID, &|writer| {
      for value in [
        self.params.patch_size,
        self.params.test_height,
        self.params.can_up,
        self.params.can_down,
      ] {
        writer.write_f32::<T>(value)?;
      }

      Ok(())
    })?;
    chunk(Self::NODES_CHUNK_ID, &|writer| {
      writer.write_u32::<T>(narrow_format_size(self.nodes.len(), "AI map nodes")?)?;

      for node in &self.nodes {
        node.write::<T>(writer)?;
      }

      Ok(())
    })?;
    chunk(Self::INTERNAL_DATA_CHUNK_ID, &|writer| {
      writer.write_f32::<T>(self.vis_radius)?;
      Ok(writer.write_u32::<T>(self.brush_size)?)
    })?;
    chunk(Self::INTERNAL_DATA2_CHUNK_ID, &|writer| {
      Ok(writer.write_f32::<T>(self.smooth_height)?)
    })?;
    chunk(Self::SNAP_OBJECTS_CHUNK_ID, &|writer| {
      writer.write_u32::<T>(narrow_format_size(self.snap_objects.len(), "snap objects")?)?;

      for name in &self.snap_objects {
        writer.write_w1251_string(name)?;
      }

      Ok(())
    })?;

    Ok(stream.buffer)
  }

  /// The part the editor saves the tool as: the scene's GUID and the tool's stream.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_bytes`] does.
  pub fn to_part<T: ByteOrder>(&self, guid: EditableLevelGuid) -> XrfResult<EditableBinaryPart> {
    Ok(EditableBinaryPart {
      guid,
      class_id: Self::CLASS_ID,
      data: self.write_to_bytes::<T>()?,
      layout: ChunkLayout::default(),
    })
  }

  /// The tool a scene's AI map tool holds, read as [`Self::read_from_part`] reads it.
  ///
  /// # Errors
  ///
  /// Returns an error when the tool is not [`Self::TOOL_NAME`], holds other than one part, or holds a text part, or as
  /// [`Self::read_from_part`] does.
  pub fn read_from_scene_tool<T: ByteOrder>(tool: &EditableSceneTool) -> XrfResult<Self> {
    match tool.parts.as_slice() {
      [EditableLevelPart::Binary(part)] if tool.name == Self::TOOL_NAME => Self::read_from_part::<T>(part),
      _ => Err(XrfError::new_parsing_error(format!(
        "Scene tool '{}' of {} parts is not the one binary part of tool '{}'",
        tool.name,
        tool.parts.len(),
        Self::TOOL_NAME
      ))),
    }
  }

  /// The tool's place in a scene: its one binary part under [`Self::TOOL_NAME`].
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_bytes`] does.
  pub fn to_scene_tool<T: ByteOrder>(&self, guid: EditableLevelGuid) -> XrfResult<EditableSceneTool> {
    Ok(EditableSceneTool {
      name: Self::TOOL_NAME.to_owned(),
      parts: vec![EditableLevelPart::Binary(self.to_part::<T>(guid)?)],
    })
  }

  fn read_vector<T: ByteOrder>(reader: &mut ChunkReader<InMemoryChunkDataSource>) -> XrfResult<Vector3d> {
    Ok(Vector3d::new(
      reader.read_f32::<T>()?,
      reader.read_f32::<T>()?,
      reader.read_f32::<T>()?,
    ))
  }
}
