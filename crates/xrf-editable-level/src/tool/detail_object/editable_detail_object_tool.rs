use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkLayout, ChunkReader, ChunkSequence, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};
use xrf_utils::narrow_format_size;

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_binary_part::EditableBinaryPart;
use crate::part::editable_level_part::EditableLevelPart;
use crate::scene::editable_scene_tool::EditableSceneTool;
use crate::tool::detail_object::editable_detail_color_index::EditableDetailColorIndex;
use crate::tool::detail_object::editable_detail_header::EditableDetailHeader;
use crate::tool::detail_object::editable_detail_object::EditableDetailObject;

/// The detail object tool's part, `detail_object.part`: its library of detail objects and the slot grid planting them,
/// as `EDetailManager::SaveStream` writes them. The Build exports it into `build.details` (`EDetailManager::Export`),
/// which the detail lighter turns into `level.details`.
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
  /// How the stream laid its chunks out, with any chunk the tool does not save, such as a converter's own, as stored.
  pub layout: ChunkLayout,
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

  /// The tool's stream as `EDetailManager::LoadStream` reads it, keeping the base texture's name whether or not the
  /// texture loads.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the editor asserts on is missing, the version is another, a chunk does not hold
  /// exactly what it is read as, the objects are not numbered from 0, the slots do not fill the header's grid, or an
  /// object does not read.
  pub fn read_from_bytes<T: ByteOrder>(data: Vec<u8>) -> XrfResult<Self> {
    let mut version: Option<u32> = None;
    let mut header: Option<EditableDetailHeader> = None;
    let mut slots: Option<Vec<[u32; Self::SLOT_WORDS]>> = None;
    let mut bbox: Option<Aabb> = None;
    let mut color_indices: Option<Vec<EditableDetailColorIndex>> = None;
    let mut flags: Option<u32> = None;
    let mut objects: Option<Vec<EditableDetailObject>> = None;
    let mut base_texture: Option<String> = None;
    let mut density: Option<f32> = None;
    let mut snap_objects: Option<Vec<String>> = None;
    let layout: ChunkLayout = ChunkLayout::read(&mut ChunkReader::from_vec(data)?, |chunk| {
      match chunk.id {
        Self::VERSION_CHUNK_ID if version.is_none() => version = Some(chunk.read_u32_payload::<T>()?),
        Self::HEADER_CHUNK_ID if header.is_none() => {
          header = Some(chunk.read_whole_payload(
            "Expect the detail header chunk to hold exactly its six values",
            |chunk| {
              Ok(EditableDetailHeader {
                version: chunk.read_u32::<T>()?,
                object_count: chunk.read_u32::<T>()?,
                offset_x: chunk.read_i32::<T>()?,
                offset_z: chunk.read_i32::<T>()?,
                size_x: chunk.read_u32::<T>()?,
                size_z: chunk.read_u32::<T>()?,
              })
            },
          )?);
        }
        Self::SLOTS_CHUNK_ID if slots.is_none() => {
          slots = Some(chunk.read_whole_payload(
            "Expect the detail slots chunk to hold exactly the slots it counts",
            |chunk| {
              let count: u32 = chunk.read_u32::<T>()?;
              let mut slots: Vec<[u32; Self::SLOT_WORDS]> =
                chunk.new_bounded_vec(count.into(), (Self::SLOT_WORDS * 4) as u64, "detail slots")?;

              for _ in 0..count {
                let mut slot: [u32; Self::SLOT_WORDS] = [0; Self::SLOT_WORDS];

                for word in &mut slot {
                  *word = chunk.read_u32::<T>()?;
                }

                slots.push(slot);
              }

              Ok(slots)
            },
          )?);
        }
        Self::BBOX_CHUNK_ID if bbox.is_none() => {
          bbox = Some(
            chunk.read_whole_payload("Expect the detail box chunk to hold exactly its box", |chunk| {
              Ok(Aabb {
                min: Self::read_vector::<T>(chunk)?,
                max: Self::read_vector::<T>(chunk)?,
              })
            })?,
          );
        }
        Self::COLOR_INDEX_CHUNK_ID if color_indices.is_none() => {
          color_indices = Some(chunk.read_whole_payload(
            "Expect the detail colour index chunk to hold exactly its indices",
            |chunk| Self::read_color_indices::<T>(chunk),
          )?);
        }
        Self::FLAGS_CHUNK_ID if flags.is_none() => flags = Some(chunk.read_u32_payload::<T>()?),
        Self::OBJECTS_CHUNK_ID if objects.is_none() => {
          objects = Some(chunk.read_numbered_children("detail object", EditableDetailObject::read::<T>)?);
        }
        Self::BASE_TEXTURE_CHUNK_ID if base_texture.is_none() => {
          base_texture = Some(chunk.read_whole_payload(
            "Expect the detail base texture chunk to hold exactly its name",
            |chunk| chunk.read_w1251_string(),
          )?);
        }
        Self::DENSITY_CHUNK_ID if density.is_none() => density = Some(chunk.read_f32_payload::<T>()?),
        Self::SNAP_OBJECTS_CHUNK_ID if snap_objects.is_none() => {
          snap_objects = Some(chunk.read_whole_payload(
            "Expect the detail snap objects chunk to hold exactly its names",
            |chunk| {
              let count: u32 = chunk.read_u32::<T>()?;

              (0..count).map(|_| chunk.read_w1251_string()).collect::<XrfResult<_>>()
            },
          )?);
        }
        _ => return Ok(false),
      }

      Ok(true)
    })?;
    let version: u32 = Self::require(version, "version")?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Detail object tool version {version} is not {}",
        Self::VERSION
      )));
    }

    let header: EditableDetailHeader = Self::require(header, "header")?;
    let slots: Vec<[u32; Self::SLOT_WORDS]> = Self::require(slots, "slots")?;

    Self::check_grid(&header, slots.len())?;

    Ok(Self {
      flags: flags.unwrap_or(Self::OBJECTS_DRAW),
      header,
      objects: objects.unwrap_or_default(),
      color_indices: Self::require(color_indices, "colour index")?,
      slots,
      bbox: Self::require(bbox, "box")?,
      base_texture,
      density: density.unwrap_or_default(),
      snap_objects: snap_objects.unwrap_or_default(),
      layout,
    })
  }

  /// A chunk `LoadStream` asserts on, `what` naming it.
  fn require<V>(value: Option<V>, what: &str) -> XrfResult<V> {
    value.ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Detail object tool has no {what} chunk, which the editor asserts on"
      ))
    })
  }

  /// The tool's stream as `EDetailManager::SaveStream` writes it, at [`Self::VERSION`], in [`Self::layout`]'s chunk
  /// order with its kept chunks.
  ///
  /// # Errors
  ///
  /// Returns an error when the slots do not fill the header's grid, a count overflows its field, or a name does not
  /// encode.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    Self::check_grid(&self.header, self.slots.len())?;

    let mut sequence: ChunkSequence = ChunkSequence::new();
    let mut chunk = |id: u32, write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult { sequence.push(id, write) };

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
      writer.write_u8(narrow_format_size::<u8>(
        self.color_indices.len(),
        "Detail object tool colour indices",
      )?)?;

      for index in &self.color_indices {
        writer.write_u32::<T>(index.color)?;
        writer.write_u8(narrow_format_size::<u8>(
          index.objects.len(),
          "Detail objects of a colour index",
        )?)?;

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

  /// The tool a scene's detail object tool holds, read as [`Self::read_from_part`] reads it.
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

  /// Whether `slots` slots fill the header's grid, `size_x * size_z`, as the level's details index them.
  fn check_grid(header: &EditableDetailHeader, slots: usize) -> XrfResult {
    let grid: u64 = u64::from(header.size_x) * u64::from(header.size_z);

    if slots as u64 == grid {
      Ok(())
    } else {
      Err(XrfError::new_parsing_error(format!(
        "Detail object tool holds {slots} slots for a grid of {} by {}, {grid} slots",
        header.size_x, header.size_z
      )))
    }
  }

  fn read_vector<T: ByteOrder>(reader: &mut ChunkReader<InMemoryChunkDataSource>) -> XrfResult<Vector3d> {
    Ok(Vector3d::new(
      reader.read_f32::<T>()?,
      reader.read_f32::<T>()?,
      reader.read_f32::<T>()?,
    ))
  }
}
