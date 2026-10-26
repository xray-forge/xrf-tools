use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, Section};

use crate::data::generic::script_to_string::ScriptToString;

/// What `se_stalker` and `se_monster` keep of an object while it is offline, written after their server class's fields
/// as two strings, each `tostring` of a value or of `nil`: the level vertex it last stood on and the logic section it
/// was running. Neither class closes them with a save marker. A compiled spawn holds `nil` for both, which is what a
/// new object starts with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineObjectState {
  pub level_vertex_id: Option<u32>,
  pub active_section: Option<String>,
}

impl OfflineObjectState {
  /// What a refused level vertex is named as.
  const LEVEL_VERTEX: &'static str = "Offline state level vertex";

  /// Import the state kept under `prefix`.
  pub fn import(section: &Section, prefix: &str) -> XrfResult<Self> {
    Ok(Self {
      level_vertex_id: ScriptToString::parse_id(
        &section.read_field::<String>(&format!("{prefix}.level_vertex_id"))?,
        Self::LEVEL_VERTEX,
      )?,
      active_section: ScriptToString::parse_text(section.read_field(&format!("{prefix}.active_section"))?),
    })
  }

  /// Export the state under `prefix`, each value as the string the script writes for it.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut Ltx) {
    ltx
      .with_section(section_name)
      .set(
        format!("{prefix}.level_vertex_id"),
        ScriptToString::format_id(self.level_vertex_id),
      )
      .set(
        format!("{prefix}.active_section"),
        ScriptToString::format_text(self.active_section.as_deref()),
      );
  }
}

impl ChunkReadWrite for OfflineObjectState {
  /// Read the two strings the script class writes.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      level_vertex_id: ScriptToString::parse_id(&reader.read_w1251_string()?, Self::LEVEL_VERTEX)?,
      active_section: ScriptToString::parse_text(reader.read_w1251_string()?),
    })
  }

  /// Write the two strings the script class reads.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&ScriptToString::format_id(self.level_vertex_id))?;
    writer.write_w1251_string(ScriptToString::format_text(self.active_section.as_deref()))?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::generic::offline_object_state::OfflineObjectState;

  fn read_back(state: &OfflineObjectState) -> XrfResult<(Vec<u8>, OfflineObjectState)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    state.write::<XRayByteOrder>(&mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: OfflineObjectState = OfflineObjectState::read::<XRayByteOrder, _>(&mut reader)?;

    reader.assert_read("Expect the whole state to be read")?;

    Ok((writer.buffer, read))
  }

  /// A compiled spawn's object has run no logic and stood nowhere yet, so both strings are `nil`.
  #[test]
  fn writes_nil_for_a_new_object() -> XrfResult {
    assert_eq!(
      read_back(&OfflineObjectState::default())?,
      (b"nil\0nil\0".to_vec(), OfflineObjectState::default())
    );

    Ok(())
  }

  #[test]
  fn reads_and_writes_a_saved_state() -> XrfResult {
    let state: OfflineObjectState = OfflineObjectState {
      level_vertex_id: Some(52235),
      active_section: Some(String::from("walker@camp")),
    };

    assert_eq!(read_back(&state)?, (b"52235\0walker@camp\0".to_vec(), state));

    Ok(())
  }

  /// `tostring` never writes a vertex as `0012`, and reading it as 12 would write it back otherwise.
  #[test]
  fn refuses_a_vertex_it_would_not_write_back() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_w1251_string("0012")?;
    writer.write_w1251_string("nil")?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let error: String = OfflineObjectState::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("a vertex the script would not write")
      .to_string();

    assert!(error.contains("Offline state level vertex '0012'"), "{error}");

    Ok(())
  }

  #[test]
  fn imports_and_exports_a_state_and_its_nils() -> XrfResult {
    let saved: OfflineObjectState = OfflineObjectState {
      level_vertex_id: Some(7),
      active_section: Some(String::from("mob_home@lair")),
    };
    let mut ltx: Ltx = Ltx::new();

    saved.export("saved", "stalker", &mut ltx);
    OfflineObjectState::default().export("new", "stalker", &mut ltx);

    let source: Ltx = Ltx::read_from_str(&String::from_utf8_lossy(&ltx.write_to_bytes()?))?;

    assert_eq!(
      OfflineObjectState::import(source.section("saved").expect("the saved section"), "stalker")?,
      saved
    );
    assert_eq!(
      OfflineObjectState::import(source.section("new").expect("the new section"), "stalker")?,
      OfflineObjectState::default()
    );

    Ok(())
  }
}
