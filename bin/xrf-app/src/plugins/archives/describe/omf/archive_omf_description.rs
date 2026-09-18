use serde::Serialize;
use xrf_error::XrfResult;
use xrf_omf::OmfFile;
use xrf_spawn::XRayByteOrder;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::omf::archive_omf_bank::ArchiveOmfBank;
use crate::plugins::archives::describe::omf::archive_omf_motion::ArchiveOmfMotion;
use crate::plugins::archives::describe::omf::archive_omf_part::ArchiveOmfPart;

/// Everything the viewer says about one motion bank.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfDescription {
  pub bank: ArchiveOmfBank,
  pub parts: Vec<ArchiveOmfPart>,
  /// Every motion, in the order the bank declares them, which is also the order the engine pairs them by.
  pub motions: Vec<ArchiveOmfMotion>,
}

impl ArchiveOmfDescription {
  /// Reads the bank an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a bank this reader can walk - a
  /// version it does not implement, or two lists that disagree about how many motions there are.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: OmfFile = OmfFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    let motions: Vec<ArchiveOmfMotion> = file
      .get_motions()
      .map(|(definition, motion)| ArchiveOmfMotion::of(definition, motion, &file.parameters.parts))
      .collect();

    Ok(Self {
      bank: ArchiveOmfBank::of(&file, &motions),
      parts: ArchiveOmfPart::of_all(&file.parameters.parts, &file.parameters.motions),
      motions,
    })
  }
}
