use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, format_path, open_export_file};

use crate::data::ogf::ogf_motion::OgfMotion;
use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;

// c++ CKinematicsAnimated
#[derive(Debug)]
pub struct OmfFile {
  pub parameters: OmfParametersChunk,
  pub motions: OmfMotionsChunk,
}

impl OmfFile {
  pub const SUPPORTED_VERSIONS: [u16; 2] = [3, 4];

  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "OMF file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived omf arrives: a volume holds no file to open.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// The route an archived omf takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_chunks::<T, _>(&chunks)
  }

  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    assert_equal(chunks.len(), 2, "Unexpected chunks count in omf file, expected 2")?;

    let parameters: OmfParametersChunk = find_required_chunk_by_id(chunks, OmfParametersChunk::CHUNK_ID)?
      .read_xr::<T, _>()
      .map_err(|error| XrfError::new_read_error(format!("Failed to read OMF parameters: {error}")))?;

    let motions: OmfMotionsChunk = find_required_chunk_by_id(chunks, OmfMotionsChunk::CHUNK_ID)?
      .read_xr::<T, _>()
      .map_err(|error| XrfError::new_read_error(format!("Failed to read OMF motions: {error}")))?;

    if parameters.motions.len() != motions.motions.len() {
      return Err(XrfError::new_parsing_error(format!(
        "Unexpected data stored in OMF file, count of motions and motions definitions mismatch: {} got, {} expected",
        parameters.motions.len(),
        motions.motions.len()
      )));
    }

    Ok(Self { parameters, motions })
  }
}

impl OmfFile {
  /// Write omf file data into provided path.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    if let Some(parent) = path.as_ref().parent() {
      fs::create_dir_all(parent)?;
    }

    self.write_to::<T>(&mut open_export_file(path)?)
  }

  /// Write omf file data into the writer.
  ///
  /// Chunks are emitted in the order used by the original game files - motions first,
  /// parameters second - so unmodified files round-trip byte for byte.
  pub fn write_to<T: ByteOrder>(&self, writer: &mut dyn Write) -> XrfResult {
    if self.parameters.motions.len() != self.motions.motions.len() {
      return Err(XrfError::new_invalid_error(format!(
        "Cannot write OMF file, count of motions and motions definitions mismatch: {} definitions, {} motions",
        self.parameters.motions.len(),
        self.motions.motions.len()
      )));
    }

    let mut motions_writer: ChunkWriter = ChunkWriter::new();
    motions_writer.write_xr::<T, _>(&self.motions)?;
    motions_writer.flush_chunk_into::<T>(writer, OmfMotionsChunk::CHUNK_ID)?;

    let mut parameters_writer: ChunkWriter = ChunkWriter::new();
    parameters_writer.write_xr::<T, _>(&self.parameters)?;
    parameters_writer.flush_chunk_into::<T>(writer, OmfParametersChunk::CHUNK_ID)?;

    Ok(())
  }
}

impl OmfFile {
  /// List names of motions stored in the file, as used for engine lookups.
  pub fn get_motion_names(&self) -> Vec<&str> {
    self.parameters.motions.iter().map(|it| it.name.as_str()).collect()
  }

  /// Motions as the engine resolves them: each definition with the key payload at its own ordinal.
  ///
  /// `motions_value::load` maps every definition name to its ordinal, then loads payload chunk `n + 1` into ordinal
  /// `n` (`xray-16/src/xrCore/Animation/SkeletonMotions.cpp`). Neither the payload label nor
  /// [`OgfMotionDefinition::motion`] joins the two, so ordinal position is the only pairing there is. Read and write
  /// both reject a file whose two lists disagree in length, so every definition here has a payload.
  pub fn get_motions(&self) -> impl Iterator<Item = (&OgfMotionDefinition, &OgfMotion)> {
    self.parameters.motions.iter().zip(self.motions.motions.iter())
  }

  /// The motion a name resolves to, with the payload at its ordinal.
  pub fn get_motion_by_name(&self, name: &str) -> Option<(&OgfMotionDefinition, &OgfMotion)> {
    self.get_motions().find(|(definition, _)| definition.name == name)
  }

  /// Count of payloads whose preserved label no longer matches the name of the motion it carries.
  ///
  /// Release playback ignores the label entirely; a non-zero count marks a bank as edited by a tool that left the
  /// prefix behind, and a `_DEBUG` engine build asserts on it.
  pub fn get_diverging_labels_count(&self) -> usize {
    self
      .get_motions()
      .filter(|(definition, motion)| !motion.has_label_matching(&definition.name))
      .count()
  }

  pub fn get_bones(&self) -> Vec<&str> {
    self
      .parameters
      .parts
      .iter()
      .flat_map(|it| it.get_bones())
      .collect::<Vec<_>>()
  }

  pub fn get_bones_count(&self) -> usize {
    self
      .parameters
      .parts
      .iter()
      .map(|it| it.get_bones().len())
      .sum::<usize>()
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice,
  };

  use crate::data::ogf::ogf_motion::OgfMotion;
  use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
  use crate::data::ogf::ogf_part::OgfPart;
  use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
  use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;
  use crate::omf::omf_file::OmfFile;

  fn new_mock(version: u16) -> OmfFile {
    OmfFile {
      parameters: OmfParametersChunk {
        version,
        parts: vec![OgfPart {
          name: String::from("default"),
          bones: vec![(String::from("bip01"), 0)],
        }],
        motions: vec![
          OgfMotionDefinition::new_mock(Vec::new()),
          OgfMotionDefinition::new_mock(Vec::new()),
        ],
      },
      motions: OmfMotionsChunk {
        motions: vec![
          OgfMotion {
            label: String::from("ak74_draw"),
            count: 2,
            flags: 1,
            remaining: vec![9, 8, 7],
          },
          OgfMotion {
            label: String::from("ak74_idle"),
            count: 4,
            flags: 0,
            remaining: vec![1, 2],
          },
        ],
      },
    }
  }

  #[test]
  fn test_write_read_file() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "write_read.omf");
    let original: OmfFile = new_mock(4);

    original.write_to_path::<XRayByteOrder, _>(&build_absolute_generated_test_resource_path(&filename))?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let chunks: Vec<ChunkReader> = ChunkReader::from_slice(file)?.read_children()?;
    let read: OmfFile = OmfFile::read_from_chunks::<XRayByteOrder, _>(&chunks)?;

    assert_eq!(read.parameters.version, original.parameters.version);
    assert_eq!(read.parameters.parts, original.parameters.parts);
    assert_eq!(read.parameters.motions, original.parameters.motions);
    assert_eq!(read.motions.motions, original.motions.motions);

    Ok(())
  }

  #[test]
  fn test_write_rejects_motions_count_mismatch() {
    let mut file: OmfFile = new_mock(4);

    file.motions.motions.pop();

    assert!(
      file.write_to::<XRayByteOrder>(&mut Vec::new()).is_err(),
      "Expect write to reject mismatched motions and definitions counts"
    );
  }

  #[test]
  fn test_read_rejects_unexpected_chunks_count() {
    let chunks: Vec<ChunkReader> = Vec::new();

    assert!(
      OmfFile::read_from_chunks::<XRayByteOrder, _>(&chunks).is_err(),
      "Expect read to reject an unexpected OMF chunks count"
    );
  }

  #[test]
  fn test_write_rejects_unsupported_version() {
    let file: OmfFile = new_mock(2);

    assert!(
      file.write_to::<XRayByteOrder>(&mut Vec::new()).is_err(),
      "Expect write to reject unsupported omf version"
    );
  }
  #[test]
  fn test_get_motion_names_reads_the_definitions() -> XrfResult {
    let mut file: OmfFile = new_mock(4);

    file.parameters.motions[0].name = String::from("ak74_draw");
    file.parameters.motions[1].name = String::from("ak74_idle");
    file.motions.motions[0].label = String::from("\u{2019}<\u{0427}");

    assert_eq!(
      file.get_motion_names(),
      vec!["ak74_draw", "ak74_idle"],
      "Expect names to come from the parameters chunk, whatever the payload labels hold"
    );

    Ok(())
  }

  #[test]
  fn test_get_motions_pairs_by_ordinal_not_by_motion_id() -> XrfResult {
    let mut file: OmfFile = new_mock(4);

    file.parameters.motions[0].name = String::from("ak74_draw");
    file.parameters.motions[1].name = String::from("ak74_idle");
    // Ids a third-party bank actually carries: neither of these addresses a payload.
    file.parameters.motions[0].motion = 50_992;
    file.parameters.motions[1].motion = 57_565;

    let paired: Vec<(&str, u32)> = file
      .get_motions()
      .map(|(definition, motion)| (definition.name.as_str(), motion.count))
      .collect();

    assert_eq!(
      paired,
      vec![("ak74_draw", 2), ("ak74_idle", 4)],
      "Expect each definition to carry the payload at its own ordinal"
    );

    assert_eq!(
      file.get_motion_by_name("ak74_idle").map(|(_, motion)| motion.count),
      Some(4)
    );
    assert!(file.get_motion_by_name("ak74_missing").is_none());

    Ok(())
  }

  #[test]
  fn test_get_diverging_labels_count_ignores_case() -> XrfResult {
    let mut file: OmfFile = new_mock(4);

    file.parameters.motions[0].name = String::from("ak74_draw");
    file.parameters.motions[1].name = String::from("ak74_idle");
    // The engine lower-cases both sides before comparing, so this one agrees.
    file.motions.motions[0].label = String::from("AK74_Draw");
    file.motions.motions[1].label = String::from("ak74_idle_old");

    assert_eq!(file.get_diverging_labels_count(), 1);

    Ok(())
  }

  #[test]
  fn test_write_read_preserves_a_non_text_label() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "non_text_label.omf");
    let mut original: OmfFile = new_mock(4);

    // Windows-1251 round-trips every byte, which is why repacking a scrambled bank is byte identical.
    original.motions.motions[0].label = String::from("\u{2019}<\u{0427}\u{2019}\u{201A}m");
    original.parameters.motions[0].motion = 50_992;

    original.write_to_path::<XRayByteOrder, _>(&build_absolute_generated_test_resource_path(&filename))?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let chunks: Vec<ChunkReader> = ChunkReader::from_slice(file)?.read_children()?;
    let read: OmfFile = OmfFile::read_from_chunks::<XRayByteOrder, _>(&chunks)?;

    assert_eq!(read.motions.motions, original.motions.motions);
    assert_eq!(read.parameters.motions[0].motion, 50_992);

    Ok(())
  }
}
