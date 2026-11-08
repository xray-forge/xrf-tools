use byteorder::ByteOrder;
use xrf_chunk::{ChunkLayout, ChunkReader, ChunkSequence, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};

use crate::level::editable_level_guid::EditableLevelGuid;

/// A tool's part saved as chunks rather than text (`EScene::SaveLTX` for a tool that cannot use an INI file): the AI
/// map, the detail objects and the wall marks. The file is the scene's GUID under [`Self::GUID_CHUNK_ID`] and the
/// tool's own stream under [`Self::DATA_CHUNK_ID`] plus its class id, as `EScene::LoadLevelPart` finds them.
///
/// The tool's stream is kept as its bytes: what each tool writes into it is read where that tool is converted.
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

  /// Reads a binary part: the GUID chunk, and the first chunk past [`Self::DATA_CHUNK_ID`] as the tool's stream.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not chunks, the GUID chunk is missing or not 16 bytes, or no data chunk is
  /// present, which the editor reports as a corrupted part.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes)?;
    let mut guid: Option<EditableLevelGuid> = None;
    let mut class_id: Option<u32> = None;
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

          guid = Some(EditableLevelGuid::read::<T>(chunk)?);
        }
        id if id >= Self::DATA_CHUNK_ID && data.is_none() => {
          class_id = Some(id - Self::DATA_CHUNK_ID);
          data = Some(chunk.read_remaining()?);
        }
        _ => return Ok(false),
      }

      Ok(true)
    })?;
    let guid: EditableLevelGuid = guid.ok_or_else(|| XrfError::new_parsing_error("Level part has no GUID chunk"))?;
    let (Some(class_id), Some(data)) = (class_id, data) else {
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
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let data_chunk_id: u32 = Self::DATA_CHUNK_ID.checked_add(self.class_id).ok_or_else(|| {
      XrfError::new_invalid_error(format!("Level part class id {} overflows its chunk id", self.class_id))
    })?;
    let mut sequence: ChunkSequence = ChunkSequence::new();

    sequence.push(Self::GUID_CHUNK_ID, |writer| self.guid.write::<T>(writer))?;
    sequence.push(data_chunk_id, |writer| {
      Ok(std::io::Write::write_all(writer, &self.data)?)
    })?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    self.layout.write::<T>(&mut writer, sequence)?;

    Ok(writer.buffer)
  }
}
