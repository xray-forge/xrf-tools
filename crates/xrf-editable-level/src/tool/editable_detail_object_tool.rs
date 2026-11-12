use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkLayout, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};
use xrf_utils::narrow_format_size;

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_binary_part::EditableBinaryPart;
use crate::part::editable_level_part::EditableLevelPart;
use crate::scene::editable_scene_tool::EditableSceneTool;
use crate::tool::editable_detail_object::EditableDetailObject;

/// The detail grid's header, `DetailHeader` (`Layers/xrRender/DetailFormat.h`), as the tool stores it: 24 bytes cast
/// onto the struct.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditableDetailHeader {
  /// `DETAIL_VERSION` once the tool has exported, else what the tool last held.
  pub version: u32,
  pub object_count: u32,
  /// The slot the level's origin falls in, and the grid's size in slots.
  pub offset_x: i32,
  pub offset_z: i32,
  pub size_x: u32,
  pub size_z: u32,
}

/// The base texture's colours the tool plants each object on, `m_ColorIndices`: a colour and the objects it plants, by
/// reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableDetailColorIndex {
  pub color: u32,
  pub objects: Vec<String>,
}

/// The detail object tool's part, `detail_object.part`: its library of detail objects and the slot grid planting them,
/// as `EDetailManager::SaveStream` (`LevelEditor/Edit/ESceneDOTools.cpp:454`) writes them into the part's data chunk.
/// The Build exports it into `build.details` (`EDetailManager::Export`), which the detail lighter turns into
/// `level.details`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableDetailObjectTool {
  /// `flObjectsDraw` and the editor's other drawing flags.
  pub flags: u32,
  pub header: EditableDetailHeader,
  /// The objects the slots plant, which a slot's corner names by its place here.
  pub objects: Vec<EditableDetailObject>,
  pub color_indices: Vec<EditableDetailColorIndex>,
  /// The grid, `size_x * size_z` slots of four words each, `DetailSlot`: height, the four corners' objects and the
  /// light, then the corners' densities.
  pub slots: Vec<[u32; Self::SLOT_WORDS]>,
  /// The box the base texture is laid over.
  pub bbox: Aabb,
  /// The texture whose colours chose the objects, where the tool keeps one.
  pub base_texture: Option<String>,
  /// Objects per square metre, `ps_r__Detail_density`, which the editor saves as its own setting.
  pub density: f32,
  /// The scene objects the base texture was laid over, by name.
  pub snap_objects: Vec<String>,
}

impl EditableDetailObjectTool {
  /// `OBJCLASS_DO`, which names the part's data chunk.
  pub const CLASS_ID: u32 = 12;

  /// The tool's `ClassName`, which names its part file.
  pub const TOOL_NAME: &'static str = "detail_object";

  /// `DETMGR_VERSION`, the only one the editor loads.
  pub const VERSION: u32 = 3;

  /// `flObjectsDraw`, the flags a new tool takes.
  pub const OBJECTS_DRAW: u32 = 1 << 28;

  /// The words of one slot, `sizeof(DetailSlot) / 4`, packed as `xrf-level`'s `LevelDetailsSlot` reads and writes
  /// them.
  pub const SLOT_WORDS: usize = 4;

  const HEADER_CHUNK_ID: u32 = 0x0000;
  const OBJECTS_CHUNK_ID: u32 = 0x0001;
  const SLOTS_CHUNK_ID: u32 = 0x0002;
  const VERSION_CHUNK_ID: u32 = 0x1000;
  const BBOX_CHUNK_ID: u32 = 0x1001;
  const BASE_TEXTURE_CHUNK_ID: u32 = 0x1002;
  const COLOR_INDEX_CHUNK_ID: u32 = 0x1003;
  const SNAP_OBJECTS_CHUNK_ID: u32 = 0x1004;
  const DENSITY_CHUNK_ID: u32 = 0x1005;
  const FLAGS_CHUNK_ID: u32 = 0x1006;

  /// # Errors
  ///
  /// Returns an error when the part is another tool's, or its data is not a stream this reads.
  pub fn read_from_part<T: ByteOrder>(part: &EditableBinaryPart) -> XrfResult<Self> {
    if part.class_id != Self::CLASS_ID {
      return Err(XrfError::new_parsing_error(format!(
        "Level part of class {} is not the detail object tool's {}",
        part.class_id,
        Self::CLASS_ID
      )));
    }

    Self::read_from_bytes::<T>(part.data.clone())
  }

  /// The tool's stream as `EDetailManager::LoadStream` reads it, the base texture's name kept whether or not the
  /// texture loads.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the editor asserts on is missing, the version is another, a chunk does not hold
  /// exactly what it counts, or an object does not read.
  pub fn read_from_bytes<T: ByteOrder>(data: Vec<u8>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(data)?.read_children()?;
    let version: u32 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u32::<T>()?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Detail object tool version {version} is not {}",
        Self::VERSION
      )));
    }

    let mut header: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::HEADER_CHUNK_ID)?;
    let header: EditableDetailHeader = EditableDetailHeader {
      version: header.read_u32::<T>()?,
      object_count: header.read_u32::<T>()?,
      offset_x: header.read_i32::<T>()?,
      offset_z: header.read_i32::<T>()?,
      size_x: header.read_u32::<T>()?,
      size_z: header.read_u32::<T>()?,
    };
    let mut slots_chunk: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::SLOTS_CHUNK_ID)?;
    let count: u32 = slots_chunk.read_u32::<T>()?;
    let mut slots: Vec<[u32; Self::SLOT_WORDS]> =
      slots_chunk.new_bounded_vec(count.into(), (Self::SLOT_WORDS * 4) as u64, "detail slots")?;

    for _ in 0..count {
      let mut slot: [u32; Self::SLOT_WORDS] = [0; Self::SLOT_WORDS];

      for word in &mut slot {
        *word = slots_chunk.read_u32::<T>()?;
      }

      slots.push(slot);
    }

    slots_chunk.assert_read("Expect the detail slots chunk to hold exactly the slots it counts")?;

    let mut bbox: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::BBOX_CHUNK_ID)?;
    let bbox: Aabb = Aabb {
      min: Self::read_vector::<T>(&mut bbox)?,
      max: Self::read_vector::<T>(&mut bbox)?,
    };
    let mut tool: Self = Self {
      flags: Self::OBJECTS_DRAW,
      header,
      objects: Vec::new(),
      color_indices: Vec::new(),
      slots,
      bbox,
      base_texture: None,
      density: 0.0,
      snap_objects: Vec::new(),
    };
    let mut has_color_indices: bool = false;

    for mut chunk in chunks {
      match chunk.id {
        Self::FLAGS_CHUNK_ID => tool.flags = chunk.read_u32::<T>()?,
        Self::OBJECTS_CHUNK_ID => {
          tool.objects = chunk
            .read_children()?
            .iter_mut()
            .map(EditableDetailObject::read::<T>)
            .collect::<XrfResult<_>>()?;
        }
        Self::COLOR_INDEX_CHUNK_ID => {
          has_color_indices = true;
          tool.color_indices = Self::read_color_indices::<T>(&mut chunk)?;
        }
        Self::BASE_TEXTURE_CHUNK_ID => tool.base_texture = Some(chunk.read_w1251_string()?),
        Self::DENSITY_CHUNK_ID => tool.density = chunk.read_f32::<T>()?,
        Self::SNAP_OBJECTS_CHUNK_ID => {
          let count: u32 = chunk.read_u32::<T>()?;

          tool.snap_objects = (0..count)
            .map(|_| chunk.read_w1251_string())
            .collect::<XrfResult<_>>()?;
        }
        _ => {}
      }
    }

    if !has_color_indices {
      return Err(XrfError::new_parsing_error(
        "Detail object tool holds no colour index chunk, which the editor asserts on",
      ));
    }

    Ok(tool)
  }

  /// The tool's stream as `EDetailManager::SaveStream` writes it, at [`Self::VERSION`].
  ///
  /// # Errors
  ///
  /// Returns an error when a count is past what its field holds, or a name does not encode.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut stream: ChunkWriter = ChunkWriter::new();
    let mut chunk = |id: u32, write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult {
      let mut writer: ChunkWriter = ChunkWriter::new();

      write(&mut writer)?;
      writer.flush_chunk_into::<T>(&mut stream.buffer, id)?;

      Ok(())
    };

    chunk(Self::VERSION_CHUNK_ID, &|writer| {
      Ok(writer.write_u32::<T>(Self::VERSION)?)
    })?;
    chunk(Self::FLAGS_CHUNK_ID, &|writer| Ok(writer.write_u32::<T>(self.flags)?))?;
    chunk(Self::HEADER_CHUNK_ID, &|writer| {
      writer.write_u32::<T>(self.header.version)?;
      writer.write_u32::<T>(self.header.object_count)?;
      writer.write_i32::<T>(self.header.offset_x)?;
      writer.write_i32::<T>(self.header.offset_z)?;
      writer.write_u32::<T>(self.header.size_x)?;
      Ok(writer.write_u32::<T>(self.header.size_z)?)
    })?;
    chunk(Self::OBJECTS_CHUNK_ID, &|writer| {
      for (index, object) in self.objects.iter().enumerate() {
        let mut entry: ChunkWriter = ChunkWriter::new();

        object.write::<T>(&mut entry)?;
        entry.flush_chunk_into::<T>(&mut writer.buffer, narrow_format_size(index, "detail objects")?)?;
      }

      Ok(())
    })?;
    chunk(Self::COLOR_INDEX_CHUNK_ID, &|writer| {
      writer.write_u8(Self::narrow_u8(self.color_indices.len(), "colour indices")?)?;

      for index in &self.color_indices {
        writer.write_u32::<T>(index.color)?;
        writer.write_u8(Self::narrow_u8(index.objects.len(), "objects of a colour index")?)?;

        for object in &index.objects {
          writer.write_w1251_string(object)?;
        }
      }

      Ok(())
    })?;
    chunk(Self::SLOTS_CHUNK_ID, &|writer| {
      writer.write_u32::<T>(narrow_format_size(self.slots.len(), "detail slots")?)?;

      for word in self.slots.as_flattened() {
        writer.write_u32::<T>(*word)?;
      }

      Ok(())
    })?;
    chunk(Self::BBOX_CHUNK_ID, &|writer| {
      for value in [self.bbox.min, self.bbox.max]
        .iter()
        .flat_map(|corner| [corner.x, corner.y, corner.z])
      {
        writer.write_f32::<T>(value)?;
      }

      Ok(())
    })?;

    if let Some(texture) = &self.base_texture {
      chunk(Self::BASE_TEXTURE_CHUNK_ID, &|writer| {
        writer.write_w1251_string(texture)?;
        Ok(())
      })?;
    }

    chunk(Self::DENSITY_CHUNK_ID, &|writer| {
      Ok(writer.write_f32::<T>(self.density)?)
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

  /// The tool a scene's detail object tool holds, read as [`Self::read_from_part`] reads it.
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

  fn read_color_indices<T: ByteOrder>(
    reader: &mut ChunkReader<InMemoryChunkDataSource>,
  ) -> XrfResult<Vec<EditableDetailColorIndex>> {
    let count: u8 = reader.read_u8()?;

    (0..count)
      .map(|_| {
        let color: u32 = reader.read_u32::<T>()?;
        let objects: u8 = reader.read_u8()?;

        Ok(EditableDetailColorIndex {
          color,
          objects: (0..objects)
            .map(|_| reader.read_w1251_string())
            .collect::<XrfResult<_>>()?,
        })
      })
      .collect()
  }

  fn read_vector<T: ByteOrder>(reader: &mut ChunkReader<InMemoryChunkDataSource>) -> XrfResult<Vector3d> {
    Ok(Vector3d::new(
      reader.read_f32::<T>()?,
      reader.read_f32::<T>()?,
      reader.read_f32::<T>()?,
    ))
  }

  fn narrow_u8(count: usize, what: &str) -> XrfResult<u8> {
    u8::try_from(count)
      .map_err(|_| XrfError::new_invalid_error(format!("A detail object tool holds {count} {what}, past a u8")))
  }
}
