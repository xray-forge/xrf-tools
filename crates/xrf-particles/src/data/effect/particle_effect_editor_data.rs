use std::io::Write;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, META_TYPE_FIELD, Section};
use xrf_utils::{assert_equal, decode_bytes_from_base64, encode_bytes_to_base64};

use crate::data::editor::particle_editor_action::ParticleEditorAction;

/// The effect as the SDK's particle editor holds it (`PED_CHUNK_EDATA`): its editor actions, kept as stored.
///
/// The engine never reads this chunk, so a library whose editor data does not decode still plays; the bytes are kept
/// whole and decoded on request through [`Self::read_actions`]. Every shipped library's decodes.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEffectEditorData {
  pub value: Vec<u8>,
}

impl ParticleEffectEditorData {
  pub const META_TYPE: &'static str = "editor_data";

  /// The editor actions the chunk holds, in the effect's order, disabled ones included.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an action list the SDK wrote: an action type or version it never used, a
  /// list that ends early, or bytes after the last action.
  pub fn read_actions<T: ByteOrder>(&self) -> XrfResult<Vec<ParticleEditorAction>> {
    ParticleEditorAction::read_list::<T, _>(&mut ChunkReader::from_bytes(&self.value)?)
  }

  /// The chunk holding `actions`, as `CPEDef::Save` writes them.
  ///
  /// # Errors
  ///
  /// Returns an error when there are more actions than the count field holds.
  pub fn from_actions<T: ByteOrder>(actions: &[ParticleEditorAction]) -> XrfResult<Self> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    ParticleEditorAction::write_list::<T>(&mut writer, actions)?;

    Ok(Self {
      value: writer.flush_raw_into_buffer()?,
    })
  }
}

impl ChunkReadWrite for ParticleEffectEditorData {
  /// Read particle effect editor data data from chunk redder.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let particle_description: Self = Self {
      value: reader.read_remaining()?,
    };

    reader.assert_read("Expect particle editor data chunk to be ended")?;

    Ok(particle_description)
  }

  /// Write particle effect description data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&self.value)?;

    Ok(())
  }
}

impl LtxImportExport for ParticleEffectEditorData {
  /// Import particle effect description data from provided path.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle effect editor data section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let meta_type: String = section.read_field(META_TYPE_FIELD)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle editor data importing",
    )?;

    Ok(Self {
      value: decode_bytes_from_base64(&section.read_field::<String>("value")?)?,
    })
  }

  /// Export particle effect editor data into provided path.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set(META_TYPE_FIELD, Self::META_TYPE)
      .set("value", encode_bytes_to_base64(&self.value));

    Ok(())
  }
}
