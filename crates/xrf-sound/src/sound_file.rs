use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use ogg::reading::PacketReader;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::SoundMetadata;
use crate::sound_file_metadata::read_sound_metadata;
use crate::sound_file_vorbis::{VorbisHeaders, decode_vorbis_stream, parse_identification_packet, read_vorbis_headers};

#[derive(Clone, Debug, PartialEq)]
pub struct SoundFile {
  pub channels: u16,
  pub metadata: SoundMetadata,
  pub sample_rate: u32,
}

impl SoundFile {
  /// Read the Ogg/Vorbis headers and X-Ray metadata of a sound file.
  ///
  /// Successful reads guarantee a structurally valid X-Ray Ogg/Vorbis header. Sounds without a
  /// recognized X-Ray comment use the engine's default source parameters. Use
  /// [`Self::read_strictly_from_path`] to fully decode the audio payload.
  pub fn read_from_path<P>(path: P) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    Self::read_from_path_with_strictness(path, false)
  }

  /// Read the headers and X-Ray metadata of a sound held in memory.
  pub fn read_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    read_xrf_sound_from(Cursor::new(bytes), false)
      .map_err(|error| XrfError::new_verify_error(format!("Failed to read sound from memory: {error}")))
  }

  /// Read and fully decode an X-Ray Ogg/Vorbis sound file.
  pub fn read_strictly_from_path<P>(path: P) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    Self::read_from_path_with_strictness(path, true)
  }

  /// Read and fully decode a sound held in memory.
  ///
  /// The strict counterpart of [`Self::read_from_bytes`], for an archived sound: a volume entry has no file to open.
  pub fn read_strictly_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    read_xrf_sound_from(Cursor::new(bytes), true)
      .map_err(|error| XrfError::new_verify_error(format!("Failed to read sound from memory: {error}")))
  }

  fn read_from_path_with_strictness<P>(path: P, is_strict: bool) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    let path: &Path = path.as_ref();

    read_xrf_sound(path, is_strict)
      .map_err(|error| XrfError::new_verify_error(format!("Failed to read sound {}: {error}", format_path(path))))
  }
}

fn read_xrf_sound(path: &Path, is_strict: bool) -> Result<SoundFile, String> {
  let file: File = File::open(path).map_err(|error| format!("Could not open sound: {error}"))?;

  read_xrf_sound_from(file, is_strict)
}

/// Shared by the path and in-memory readers so the two cannot disagree about what a sound file is.
fn read_xrf_sound_from<R>(source: R, is_strict: bool) -> Result<SoundFile, String>
where
  R: Read + Seek,
{
  let mut reader: PacketReader<R> = PacketReader::new(source);

  let headers: VorbisHeaders = read_vorbis_headers(&mut reader)?;
  let (channels, sample_rate): (u16, u32) = parse_identification_packet(&headers.identification)?;
  let metadata: SoundMetadata = read_sound_metadata(&headers.comment)?;

  if is_strict {
    decode_vorbis_stream(&mut reader, &headers)?;
  }

  Ok(SoundFile {
    channels,
    metadata,
    sample_rate,
  })
}
