use xrf_chunk::{ChunkLayout, ChunkReader, ChunkSequence, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::tool::ai_map::editable_ai_map_tool::EditableAiMapTool;
use crate::tool::detail_object::editable_detail_object_tool::EditableDetailObjectTool;
use crate::tool::wallmark::editable_wallmark_tool::EditableWallmarkTool;

/// A tool's part saved as chunks rather than text (`EScene::SaveLTX`): the AI map, detail objects and wall marks. The
/// file is the scene's GUID under [`Self::GUID_CHUNK_ID`] and the tool's stream under [`Self::DATA_CHUNK_ID`] plus its
/// class id.
///
/// The stream is kept as bytes, which the tool's own type reads ([`EditableAiMapTool`], [`EditableDetailObjectTool`],
/// [`EditableWallmarkTool`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableBinaryPart {
  pub guid: EditableLevelGuid,
  /// The tool's `ObjClassID`, which names its data chunk.
  pub class_id: u32,
  /// The tool's `SaveStream` output, as stored.
  pub data: Vec<u8>,
  /// How the file laid its chunks out, and any chunk neither field takes.
  pub layout: ChunkLayout,
}

impl EditableBinaryPart {
  /// `CHUNK_TOOLS_GUID`.
  pub const GUID_CHUNK_ID: u32 = 0x7000;

  /// `CHUNK_TOOLS_DATA`, to which a tool's class id is added.
  pub const DATA_CHUNK_ID: u32 = 0x8000;

  /// Bytes the GUID chunk holds.
  pub const GUID_SIZE: u64 = 16;

  /// The class a tool saves its binary part as, by the tool's name, which names its files; none for a tool saving text.
  pub fn get_tool_class_id(tool: &str) -> Option<u32> {
    match tool {
      EditableAiMapTool::TOOL_NAME => Some(EditableAiMapTool::CLASS_ID),
      EditableDetailObjectTool::TOOL_NAME => Some(EditableDetailObjectTool::CLASS_ID),
      EditableWallmarkTool::TOOL_NAME => Some(EditableWallmarkTool::CLASS_ID),
      _ => None,
    }
  }

  /// Reads a binary part as `EScene::LoadLevelPart` does, the stream from the data chunk of `class_id`. With no class,
  /// or none of it held, the first data chunk is the stream, so a part saved for another tool still reads and
  /// [`Self::class_id`] tells it apart; other data chunks stay in the layout.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not chunks, the GUID chunk is missing or not 16 bytes, or no data chunk is
  /// present, which the editor reports as a corrupted part.
  pub fn read_from_bytes(bytes: Vec<u8>, class_id: Option<u32>) -> XrfResult<Self> {
    let wanted: Option<u32> = class_id.and_then(|class_id| Self::DATA_CHUNK_ID.checked_add(class_id));
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes)?;
    let is_wanted_held: bool = wanted.is_some_and(|wanted| {
      reader
        .get_children_cloned()
        .is_ok_and(|chunks| chunks.iter().any(|chunk| chunk.id == wanted))
    });
    let mut guid: Option<EditableLevelGuid> = None;
    let mut found_class: Option<u32> = None;
    let mut data: Option<Vec<u8>> = None;
    let layout: ChunkLayout = ChunkLayout::read(&mut reader, |chunk| {
      match chunk.id {
        Self::GUID_CHUNK_ID if guid.is_none() => {
          if chunk.read_bytes_remain() != Self::GUID_SIZE {
            return Err(XrfError::new_parsing_error(format!(
              "Level part GUID chunk holds {} bytes, expected {}",
              chunk.read_bytes_remain(),
              Self::GUID_SIZE
            )));
          }

          guid = Some(EditableLevelGuid::read(chunk)?);
        }
        id if id >= Self::DATA_CHUNK_ID && data.is_none() && (!is_wanted_held || Some(id) == wanted) => {
          found_class = Some(id - Self::DATA_CHUNK_ID);
          data = Some(chunk.read_remaining()?);
        }
        _ => return Ok(false),
      }

      Ok(true)
    })?;
    let guid: EditableLevelGuid = guid.ok_or_else(|| XrfError::new_parsing_error("Level part has no GUID chunk"))?;
    let (Some(class_id), Some(data)) = (found_class, data) else {
      return Err(XrfError::new_parsing_error("Level part has no tool data chunk"));
    };

    Ok(Self {
      guid,
      class_id,
      data,
      layout,
    })
  }

  /// The part as the editor frames it: the GUID chunk, then the tool's data chunk, in the order the file stored them.
  ///
  /// # Errors
  ///
  /// Returns an error when the class id would overflow the data chunk's id or the data is larger than a chunk holds.
  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    let data_chunk_id: u32 = Self::DATA_CHUNK_ID.checked_add(self.class_id).ok_or_else(|| {
      XrfError::new_invalid_error(format!("Level part class id {} overflows its chunk id", self.class_id))
    })?;
    let mut sequence: ChunkSequence = ChunkSequence::new();

    sequence.push(Self::GUID_CHUNK_ID, |writer| self.guid.write(writer))?;
    sequence.push(data_chunk_id, |writer| {
      Ok(std::io::Write::write_all(writer, &self.data)?)
    })?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    self.layout.write(&mut writer, sequence)?;

    Ok(writer.buffer)
  }
}
