use std::io::{Read, Write};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_error::XrfResult;
use xrf_ini::IniSection;

/// A scene's identity, the editor's `xrGUID`: every part of a scene carries its scene's, and the editor skips a part
/// whose GUID differs (`EScene::LoadLevelPartLTX`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EditableLevelGuid {
  pub g0: u64,
  pub g1: u64,
}

impl EditableLevelGuid {
  /// The section `xrGUID::SaveLTX` writes a GUID under, in the `.level` and in every text part.
  pub const SECTION: &str = "guid";

  /// Reads the `guid_g0` and `guid_g1` fields of a section.
  ///
  /// # Errors
  ///
  /// Returns an error when either is missing or not an unsigned 64-bit integer.
  pub fn read_from_section(section: &IniSection) -> XrfResult<Self> {
    Ok(Self {
      g0: section.get_parsed::<u64>("guid_g0")?,
      g1: section.get_parsed::<u64>("guid_g1")?,
    })
  }

  pub fn write_to_section(&self, section: &mut IniSection) {
    section.set("guid_g0", self.g0.to_string());
    section.set("guid_g1", self.g1.to_string());
  }

  /// Reads the two words a binary part stores.
  ///
  /// # Errors
  ///
  /// Returns an error when fewer than 16 bytes remain.
  pub fn read<T: ByteOrder>(reader: &mut impl Read) -> XrfResult<Self> {
    Ok(Self {
      g0: reader.read_u64::<T>()?,
      g1: reader.read_u64::<T>()?,
    })
  }

  /// # Errors
  ///
  /// Returns an error when the writer fails.
  pub fn write<T: ByteOrder>(&self, writer: &mut impl Write) -> XrfResult {
    writer.write_u64::<T>(self.g0)?;
    writer.write_u64::<T>(self.g1)?;

    Ok(())
  }
}
