use std::io::Write;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, META_TYPE_FIELD, Section, read_ltx_field};
use xrf_utils::{assert_equal, decode_bytes_from_base64, encode_bytes_to_base64};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEffectEditorData {
  pub value: Vec<u8>,
}

impl ParticleEffectEditorData {
  pub const META_TYPE: &'static str = "editor_data";
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

    let meta_type: String = read_ltx_field(META_TYPE_FIELD, section)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle editor data importing",
    )?;

    Ok(Self {
      value: decode_bytes_from_base64(&read_ltx_field::<String>("value", section)?)?,
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
