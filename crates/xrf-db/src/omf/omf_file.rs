use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_equal, open_export_file};

use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;

// c++ CKinematicsAnimated
#[derive(Debug, Serialize, Deserialize)]
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
        path.as_ref().display(),
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
  /// Read only list of motions specifically and skip other data parts.
  pub fn read_motions_from_path<T: ByteOrder, P: AsRef<Path>>(path: P) -> XrfResult<Vec<String>> {
    Self::read_motions_from_file::<T>(File::open(path)?)
  }

  pub fn read_motions_from_file<T: ByteOrder>(file: File) -> XrfResult<Vec<String>> {
    Self::read_motions_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Lists motion names from a chunk reader over any data source.
  ///
  /// The route an archived omf takes: a volume entry has no file, so only its decompressed bytes are available.
  pub fn read_motions_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<String>> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    log::info!(
      "Reading omf file motions, {} chunks, {} bytes",
      chunks.len(),
      reader.read_bytes_len(),
    );

    Ok(
      find_required_chunk_by_id(&chunks, OmfMotionsChunk::CHUNK_ID)?
        .read_xr::<T, OmfMotionsChunk>()?
        .motions
        .iter()
        .map(|it| it.name.clone())
        .collect(),
    )
  }
}

impl OmfFile {
  /// List names of motions stored in the file, as used for engine lookups.
  pub fn get_motion_names(&self) -> Vec<&str> {
    self.parameters.motions.iter().map(|it| it.name.as_str()).collect()
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
            name: String::from("ak74_draw"),
            count: 2,
            flags: 1,
            remaining: vec![9, 8, 7],
          },
          OgfMotion {
            name: String::from("ak74_idle"),
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
}
