use serde::Serialize;
use xrf_db::{DetailModel, LevelDetailsFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::detail::archive_detail_model::ArchiveDetailModel;

/// One object of a level's detail library, and how much of the level it is planted across.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDetailEntry {
  /// Position in the library, which is what a slot's six bits address.
  pub index: u32,
  /// Slot corners planting this object across the whole grid.
  pub planted_corners: u64,
  pub model: ArchiveDetailModel,
}

/// Everything the viewer says about a level's detail layer.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDetailLibraryDescription {
  pub version: u32,
  pub size_x: u32,
  pub size_z: u32,
  /// Cells the grid holds, which is `size_x * size_z`.
  pub slots: u64,
  /// Cells planting at least one object, which is what says how much of the level is actually dressed.
  pub planted_slots: u64,
  /// Ground the grid covers along each axis, in engine units, which are metres.
  pub covers_x: f32,
  pub covers_z: f32,
  /// The library, in the order the file numbers it.
  pub entries: Vec<ArchiveDetailEntry>,
}

impl ArchiveDetailLibraryDescription {
  /// Reads the detail layer an entry holds and resolves what it names against the subject being browsed.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a detail library this reader can
  /// walk - a version it does not implement, or a grid the header does not account for.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;
    let (covers_x, covers_z): (f32, f32) = file.get_covered_meters();
    let usage: Vec<u64> = file.get_object_usage();

    Ok(Self {
      version: file.header.version,
      size_x: file.header.size_x,
      size_z: file.header.size_z,
      slots: file.get_slots_count(),
      planted_slots: file.get_planted_slots_count(),
      covers_x,
      covers_z,
      entries: file
        .objects
        .iter()
        .enumerate()
        .map(|(index, model): (usize, &DetailModel)| ArchiveDetailEntry {
          index: index as u32,
          planted_corners: usage.get(index).copied().unwrap_or_default(),
          model: ArchiveDetailModel::of(source, model),
        })
        .collect(),
    })
  }
}
