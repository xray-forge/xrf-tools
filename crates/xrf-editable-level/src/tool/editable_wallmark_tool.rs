use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkLayout, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::narrow_format_size;

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_binary_part::EditableBinaryPart;
use crate::part::editable_level_part::EditableLevelPart;
use crate::scene::editable_scene_tool::EditableSceneTool;
use crate::tool::editable_wallmark::EditableWallmark;

/// Every decal of one shader and texture, `ESceneWallmarkTool::wm_slot`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWallmarkSlot {
  /// The blender the decals draw through, a definition inside `shaders.xr`.
  pub shader: String,
  pub texture: String,
  pub marks: Vec<EditableWallmark>,
}

/// The wall mark tool's part, `wallmark.part`: its brush settings and its decals in slots, as
/// `ESceneWallmarkTool::SaveStream` writes them into the part's data chunk. The editor exports the decals into
/// `level.wallmarks`, and hands their fans to the compiler as faces too.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWallmarkTool {
  /// `flDrawWallmark` and `flAxisAlign`.
  pub flags: u32,
  /// The brush a new decal is shot with: its size, turn, shader and texture.
  pub mark_width: f32,
  pub mark_height: f32,
  pub mark_rotation: f32,
  pub shader: String,
  pub texture: String,
  /// The slots in their stored order, an empty one kept as stored; the editor drops it on load.
  pub slots: Vec<EditableWallmarkSlot>,
}

impl EditableWallmarkTool {
  /// `OBJCLASS_WM`, which names the part's data chunk.
  pub const CLASS_ID: u32 = 14;

  /// The tool's `ClassName`, which names its part file.
  pub const TOOL_NAME: &'static str = "wallmark";

  /// `WM_VERSION`, which the editor writes; it reads [`Self::OLD_VERSION`] too.
  pub const VERSION: u16 = 4;

  /// The version before decals recorded their size and turn, whose brush shader the editor replaces with
  /// [`Self::DEFAULT_SHADER`].
  pub const OLD_VERSION: u16 = 3;

  /// `flDrawWallmark`, the flags a new tool takes.
  pub const DRAW: u32 = 1;

  /// The brush shader a new tool takes.
  pub const DEFAULT_SHADER: &'static str = "effects\\wallmarkmult";

  const VERSION_CHUNK_ID: u32 = 1;
  const FLAGS_CHUNK_ID: u32 = 2;
  const PARAMS_CHUNK_ID: u32 = 3;
  const OLD_ITEMS_CHUNK_ID: u32 = 4;
  const ITEMS_CHUNK_ID: u32 = 5;

  /// A tool of no decal, with the brush a new one takes.
  pub fn new(slots: Vec<EditableWallmarkSlot>) -> Self {
    Self {
      flags: Self::DRAW,
      mark_width: 1.0,
      mark_height: 1.0,
      mark_rotation: 0.0,
      shader: Self::DEFAULT_SHADER.to_owned(),
      texture: String::new(),
      slots,
    }
  }

  /// # Errors
  ///
  /// Returns an error when the part is another tool's, or its data is not a stream this reads.
  pub fn read_from_part<T: ByteOrder>(part: &EditableBinaryPart) -> XrfResult<Self> {
    if part.class_id != Self::CLASS_ID {
      return Err(XrfError::new_parsing_error(format!(
        "Level part of class {} is not the wall mark tool's {}",
        part.class_id,
        Self::CLASS_ID
      )));
    }

    Self::read_from_bytes::<T>(part.data.clone())
  }

  /// The tool's stream as `ESceneWallmarkTool::LoadStream` reads it, from version 3 or 4: its decals from the older
  /// chunk where one is stored, which keeps no size or turn, else from the current one.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the editor asserts on is missing, the version is another, or a slot does not hold
  /// exactly what it counts.
  pub fn read_from_bytes<T: ByteOrder>(data: Vec<u8>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(data)?.read_children()?;
    let version: u16 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u16::<T>()?;

    if version != Self::VERSION && version != Self::OLD_VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Wall mark tool version {version} is neither {} nor {}",
        Self::OLD_VERSION,
        Self::VERSION
      )));
    }

    let flags: u32 = ChunkReader::find_required_by_id(&chunks, Self::FLAGS_CHUNK_ID)?.read_u32::<T>()?;
    let mut params: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::PARAMS_CHUNK_ID)?;
    let [mark_width, mark_height, mark_rotation] = [
      params.read_f32::<T>()?,
      params.read_f32::<T>()?,
      params.read_f32::<T>()?,
    ];
    let stored_shader: String = params.read_w1251_string()?;
    let texture: String = params.read_w1251_string()?;
    // The older decal chunk wins wherever it stands, whatever the version, as `LoadStream` opens it first.
    let old_items: Option<usize> = chunks.iter().position(|chunk| chunk.id == Self::OLD_ITEMS_CHUNK_ID);
    let items: Option<usize> = old_items.or_else(|| chunks.iter().position(|chunk| chunk.id == Self::ITEMS_CHUNK_ID));
    let slots: Vec<EditableWallmarkSlot> = match items.and_then(|index| chunks.into_iter().nth(index)) {
      Some(mut items) => items
        .read_children()?
        .into_iter()
        .map(|mut slot| Self::read_slot::<T>(&mut slot, old_items.is_none()))
        .collect::<XrfResult<Vec<EditableWallmarkSlot>>>()?,
      None => Vec::new(),
    };

    Ok(Self {
      flags,
      mark_width,
      mark_height,
      mark_rotation,
      shader: if version == Self::OLD_VERSION {
        Self::DEFAULT_SHADER.to_owned()
      } else {
        stored_shader
      },
      texture,
      slots,
    })
  }

  /// The tool's stream as `ESceneWallmarkTool::SaveStream` writes it, at [`Self::VERSION`].
  ///
  /// # Errors
  ///
  /// Returns an error when a count is past what its `u32` holds.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut stream: ChunkWriter = ChunkWriter::new();
    let mut version: ChunkWriter = ChunkWriter::new();
    let mut flags: ChunkWriter = ChunkWriter::new();
    let mut params: ChunkWriter = ChunkWriter::new();
    let mut items: ChunkWriter = ChunkWriter::new();

    version.write_u16::<T>(Self::VERSION)?;
    flags.write_u32::<T>(self.flags)?;
    params.write_f32::<T>(self.mark_width)?;
    params.write_f32::<T>(self.mark_height)?;
    params.write_f32::<T>(self.mark_rotation)?;
    params.write_w1251_string(&self.shader)?;
    params.write_w1251_string(&self.texture)?;

    for (index, slot) in self.slots.iter().enumerate() {
      let mut writer: ChunkWriter = ChunkWriter::new();

      writer.write_u32::<T>(narrow_format_size(slot.marks.len(), "wall marks")?)?;

      if !slot.marks.is_empty() {
        writer.write_w1251_string(&slot.shader)?;
        writer.write_w1251_string(&slot.texture)?;

        for mark in &slot.marks {
          mark.write::<T>(&mut writer)?;
        }
      }

      writer.flush_chunk_into::<T>(&mut items.buffer, narrow_format_size(index, "wall mark slots")?)?;
    }

    version.flush_chunk_into::<T>(&mut stream.buffer, Self::VERSION_CHUNK_ID)?;
    flags.flush_chunk_into::<T>(&mut stream.buffer, Self::FLAGS_CHUNK_ID)?;
    params.flush_chunk_into::<T>(&mut stream.buffer, Self::PARAMS_CHUNK_ID)?;
    items.flush_chunk_into::<T>(&mut stream.buffer, Self::ITEMS_CHUNK_ID)?;

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

  /// The tool a scene's wall mark tool holds, read as [`Self::read_from_part`] reads it.
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

  fn read_slot<T: ByteOrder>(
    reader: &mut ChunkReader<InMemoryChunkDataSource>,
    recorded: bool,
  ) -> XrfResult<EditableWallmarkSlot> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut slot: EditableWallmarkSlot = EditableWallmarkSlot {
      shader: String::new(),
      texture: String::new(),
      marks: Vec::new(),
    };

    if count > 0 {
      slot.shader = reader.read_w1251_string()?;
      slot.texture = reader.read_w1251_string()?;
      slot.marks = reader.new_bounded_vec(count.into(), EditableWallmark::OLD_FIXED_SIZE, "wall marks")?;

      for _ in 0..count {
        slot.marks.push(EditableWallmark::read::<T, _>(reader, recorded)?);
      }
    }

    reader.assert_read("Expect a wall mark slot to hold exactly the decals it counts")?;

    Ok(slot)
  }
}
