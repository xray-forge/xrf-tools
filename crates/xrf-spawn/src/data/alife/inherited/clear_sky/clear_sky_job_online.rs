use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

/// Whether a Clear Sky stalker or monster is kept online or offline by its job, `job_online` of `se_stalker` and
/// `se_monster`: a byte, followed by the condition list when the byte is 3. The scripts read any byte past 2 as a
/// condition list and write 3, so only 0 to 3 are accepted. A spawn holds `Unset`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ClearSkyJobOnline {
  /// `true`, written as 0.
  Online,
  /// `false`, written as 1.
  Offline,
  /// `nil`, written as 2: the job does not decide.
  #[default]
  Unset,
  /// A condition list deciding it, written as 3 and the list's text.
  Condlist(String),
}

impl ClearSkyJobOnline {
  const ONLINE: u8 = 0;
  const OFFLINE: u8 = 1;
  const UNSET: u8 = 2;
  const CONDLIST: u8 = 3;

  /// The LTX values of the kinds; a condition list's text goes under its own key.
  const ONLINE_TEXT: &'static str = "online";
  const OFFLINE_TEXT: &'static str = "offline";
  const UNSET_TEXT: &'static str = "nil";
  const CONDLIST_TEXT: &'static str = "condlist";

  /// Import the value kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    let kind: String = section.read_field(&format!("{prefix}.job_online"))?;

    Ok(match kind.as_str() {
      Self::ONLINE_TEXT => Self::Online,
      Self::OFFLINE_TEXT => Self::Offline,
      Self::UNSET_TEXT => Self::Unset,
      Self::CONDLIST_TEXT => Self::Condlist(section.read_field(&format!("{prefix}.job_online_condlist"))?),
      other => {
        return Err(XrfError::new_parsing_error(format!(
          "Job online '{other}' is none of online, offline, nil or condlist"
        )));
      }
    })
  }

  /// Export the value under `prefix`, a condition list's text under its own key.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    let mut section = ltx.with_section(section_name);
    let kind: &str = match self {
      Self::Online => Self::ONLINE_TEXT,
      Self::Offline => Self::OFFLINE_TEXT,
      Self::Unset => Self::UNSET_TEXT,
      Self::Condlist(condlist) => {
        section.set(format!("{prefix}.job_online_condlist"), condlist);

        Self::CONDLIST_TEXT
      }
    };

    section.set(format!("{prefix}.job_online"), kind);
  }
}

impl ChunkFieldReadWrite for ClearSkyJobOnline {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(match reader.read_u8()? {
      Self::ONLINE => Self::Online,
      Self::OFFLINE => Self::Offline,
      Self::UNSET => Self::Unset,
      Self::CONDLIST => Self::Condlist(reader.read_w1251_string()?),
      other => {
        return Err(XrfError::new_invalid_error(format!(
          "Job online kind {other} is read as a condition list but written back as {}",
          Self::CONDLIST
        )));
      }
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    match self {
      Self::Online => writer.write_u8(Self::ONLINE),
      Self::Offline => writer.write_u8(Self::OFFLINE),
      Self::Unset => writer.write_u8(Self::UNSET),
      Self::Condlist(condlist) => {
        writer.write_u8(Self::CONDLIST)?;
        writer.write_w1251_string(condlist)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::inherited::clear_sky::clear_sky_job_online::ClearSkyJobOnline;

  #[test]
  fn reads_and_writes_each_kind() -> XrfResult {
    for (value, bytes) in [
      (ClearSkyJobOnline::Online, b"\x00".to_vec()),
      (ClearSkyJobOnline::Offline, b"\x01".to_vec()),
      (ClearSkyJobOnline::Unset, b"\x02".to_vec()),
      (
        ClearSkyJobOnline::Condlist(String::from("{+info} true, false")),
        b"\x03{+info} true, false\0".to_vec(),
      ),
    ] {
      let mut writer: ChunkWriter = ChunkWriter::new();

      value.write::<XRayByteOrder>(&mut writer)?;

      assert_eq!(writer.buffer, bytes);

      let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

      assert_eq!(ClearSkyJobOnline::read::<XRayByteOrder, _>(&mut reader)?, value);
      assert!(reader.is_ended());

      let mut ltx: Ltx = Ltx::new();

      value.export("object", "stalker", &mut ltx);

      assert_eq!(
        ClearSkyJobOnline::import(ltx.section("object").expect("the section"), "stalker")?,
        value
      );
    }

    Ok(())
  }

  /// The scripts read a kind past 3 as a condition list and would write 3 for it.
  #[test]
  fn refuses_a_kind_it_would_write_back_otherwise() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"\x04x\0")?;
    let error: String = ClearSkyJobOnline::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("kind 4")
      .to_string();

    assert!(error.contains("Job online kind 4"), "{error}");

    Ok(())
  }
}
