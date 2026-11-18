use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkLayout, ChunkReader, ChunkSequence, ChunkWriter, InMemoryChunkDataSource};
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
/// `ESceneWallmarkTool::SaveStream` writes them. The editor exports the decals into `level.wallmarks`, and hands their
/// fans to the compiler as faces too.
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
  /// How the stream laid its chunks out, with any chunk the tool does not save, such as a converter's own, as stored.
  pub layout: ChunkLayout,
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

  /// A tool of `slots`, with the brush a new tool takes.
  pub fn new(slots: Vec<EditableWallmarkSlot>) -> Self {
    Self {
      flags: Self::DRAW,
      mark_width: 1.0,
      mark_height: 1.0,
      mark_rotation: 0.0,
      shader: Self::DEFAULT_SHADER.to_owned(),
      texture: String::new(),
      slots,
      layout: ChunkLayout::default(),
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

  /// The tool's stream as `ESceneWallmarkTool::LoadStream` reads it, at [`Self::OLD_VERSION`] or [`Self::VERSION`].
  /// The decals come from the older chunk where one is stored, which keeps no size or turn, else from the current one;
  /// the other decal chunk is passed over, as the editor does.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the editor asserts on is missing, the version is another, a chunk does not hold
  /// exactly what it is read as, the slots are not numbered from 0, or a slot does not hold exactly what it counts.
  pub fn read_from_bytes<T: ByteOrder>(data: Vec<u8>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut flags: Option<u32> = None;
    let mut params: Option<(f32, f32, f32, String, String)> = None;
    let mut old_items: Option<Vec<u8>> = None;
    let mut items: Option<Vec<u8>> = None;
    let layout: ChunkLayout = ChunkLayout::read(&mut ChunkReader::from_vec(data)?, |chunk| {
      match chunk.id {
        Self::VERSION_CHUNK_ID if version.is_none() => version = Some(chunk.read_u16_payload::<T>()?),
        Self::FLAGS_CHUNK_ID if flags.is_none() => flags = Some(chunk.read_u32_payload::<T>()?),
        Self::PARAMS_CHUNK_ID if params.is_none() => {
          params = Some(chunk.read_whole_payload(
            "Expect the wall mark params chunk to hold exactly its brush",
            |chunk| {
              Ok((
                chunk.read_f32::<T>()?,
                chunk.read_f32::<T>()?,
                chunk.read_f32::<T>()?,
                chunk.read_w1251_string()?,
                chunk.read_w1251_string()?,
              ))
            },
          )?);
        }
        Self::OLD_ITEMS_CHUNK_ID if old_items.is_none() => old_items = Some(chunk.read_remaining()?),
        Self::ITEMS_CHUNK_ID if items.is_none() => items = Some(chunk.read_remaining()?),
        _ => return Ok(false),
      }

      Ok(true)
    })?;
    let version: u16 = Self::require(version, "version")?;

    if version != Self::VERSION && version != Self::OLD_VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Wall mark tool version {version} is neither {} nor {}",
        Self::OLD_VERSION,
        Self::VERSION
      )));
    }

    let (mark_width, mark_height, mark_rotation, stored_shader, texture) = Self::require(params, "params")?;
    // The older decal chunk wins wherever it stands, whatever the version, as `LoadStream` opens it first.
    let is_current: bool = old_items.is_none();
    // A tool of no decal stores its decal chunk empty, as the editor writes it.
    let slots: Vec<EditableWallmarkSlot> = match old_items.or(items).filter(|bytes| !bytes.is_empty()) {
      Some(bytes) => ChunkReader::from_vec(bytes)?
        .read_numbered_children("wall mark slot", |slot| Self::read_slot::<T>(slot, is_current))?,
      None => Vec::new(),
    };

    Ok(Self {
      flags: Self::require(flags, "flags")?,
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
      layout,
    })
  }

  /// A chunk `LoadStream` asserts on, `what` naming it.
  fn require<V>(value: Option<V>, what: &str) -> XrfResult<V> {
    value.ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Wall mark tool has no {what} chunk, which the editor asserts on"
      ))
    })
  }

  /// The tool's stream as `ESceneWallmarkTool::SaveStream` writes it, at [`Self::VERSION`], in [`Self::layout`]'s
  /// chunk order with its kept chunks.
  ///
  /// # Errors
  ///
  /// Returns an error when a count overflows its `u32`.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut sequence: ChunkSequence = ChunkSequence::new();

    sequence.push(Self::VERSION_CHUNK_ID, |writer| {
      Ok(writer.write_u16::<T>(Self::VERSION)?)
    })?;
    sequence.push(Self::FLAGS_CHUNK_ID, |writer| Ok(writer.write_u32::<T>(self.flags)?))?;
    sequence.push(Self::PARAMS_CHUNK_ID, |writer| {
      writer.write_f32::<T>(self.mark_width)?;
      writer.write_f32::<T>(self.mark_height)?;
      writer.write_f32::<T>(self.mark_rotation)?;
      writer.write_w1251_string(&self.shader)?;
      writer.write_w1251_string(&self.texture)?;

      Ok(())
    })?;
    sequence.push(Self::ITEMS_CHUNK_ID, |items| {
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

      Ok(())
    })?;

    let mut stream: ChunkWriter = ChunkWriter::new();

    self.layout.write::<T>(&mut stream, sequence)?;

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
  /// Returns an error when the tool is not [`Self::TOOL_NAME`] or holds other than one binary part, or as
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
