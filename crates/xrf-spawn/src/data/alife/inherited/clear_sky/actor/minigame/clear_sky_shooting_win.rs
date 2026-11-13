use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

/// The shooting minigame's `win` parameter, a boolean or a count by the game played: a kind byte, 0 for a `w_bool` and
/// 1 for a `u16`. The script reads any other kind as a count and writes 1, so only 0 and 1 are accepted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ClearSkyShootingWin {
  Flag(u8),
  Count(u16),
}

impl ClearSkyShootingWin {
  const FLAG: u8 = 0;
  const COUNT: u8 = 1;

  const FLAG_TEXT: &'static str = "flag";
  const COUNT_TEXT: &'static str = "count";

  /// Import the value kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    let kind: String = section.read_field(&format!("{prefix}.win_kind"))?;
    let key: String = format!("{prefix}.win");

    Ok(match kind.as_str() {
      Self::FLAG_TEXT => Self::Flag(section.read_field(&key)?),
      Self::COUNT_TEXT => Self::Count(section.read_field(&key)?),
      other => {
        return Err(XrfError::new_parsing_error(format!(
          "Shooting win kind '{other}' is neither flag nor count"
        )));
      }
    })
  }

  /// Export the value under `prefix`, its kind beside it.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    let mut section = ltx.with_section(section_name);

    match self {
      Self::Flag(flag) => section
        .set(format!("{prefix}.win_kind"), Self::FLAG_TEXT)
        .set(format!("{prefix}.win"), flag),
      Self::Count(count) => section
        .set(format!("{prefix}.win_kind"), Self::COUNT_TEXT)
        .set(format!("{prefix}.win"), count),
    };
  }
}

impl ChunkFieldReadWrite for ClearSkyShootingWin {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    match reader.read_u8()? {
      Self::FLAG => Ok(Self::Flag(reader.read_u8()?)),
      Self::COUNT => Ok(Self::Count(reader.read_u16::<T>()?)),
      other => Err(XrfError::new_invalid_error(format!(
        "Shooting win kind {other} is read as a count but written back as {}",
        Self::COUNT
      ))),
    }
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    match self {
      Self::Flag(flag) => {
        writer.write_u8(Self::FLAG)?;
        writer.write_u8(*flag)
      }
      Self::Count(count) => {
        writer.write_u8(Self::COUNT)?;
        writer.write_u16::<T>(*count)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::inherited::clear_sky::actor::minigame::clear_sky_shooting_win::ClearSkyShootingWin;

  #[test]
  fn reads_and_writes_each_kind() -> XrfResult {
    for (value, bytes) in [
      (ClearSkyShootingWin::Flag(1), b"\x00\x01".to_vec()),
      (ClearSkyShootingWin::Count(300), b"\x01\x2c\x01".to_vec()),
    ] {
      let mut writer: ChunkWriter = ChunkWriter::new();

      value.write::<XRayByteOrder>(&mut writer)?;

      assert_eq!(writer.buffer, bytes);
      assert_eq!(
        ClearSkyShootingWin::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(
          &bytes
        )?)?,
        value
      );

      let mut ltx: Ltx = Ltx::new();

      value.export("object", "shooting", &mut ltx);

      assert_eq!(
        ClearSkyShootingWin::import(ltx.section("object").expect("the section"), "shooting")?,
        value
      );
    }

    Ok(())
  }

  #[test]
  fn refuses_a_kind_it_would_write_back_otherwise() -> XrfResult {
    let error: String = ClearSkyShootingWin::read::<XRayByteOrder, _>(
      &mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(b"\x02\0\0")?,
    )
    .expect_err("kind 2")
    .to_string();

    assert!(error.contains("Shooting win kind 2"), "{error}");

    Ok(())
  }
}
