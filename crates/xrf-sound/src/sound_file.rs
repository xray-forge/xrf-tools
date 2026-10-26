use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use std::time::Duration;

use ogg::reading::PacketReader;
use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::sound_metadata::SoundMetadata;
use crate::vorbis::vorbis_stream::VorbisStream;
use crate::xray::xray_sound_comment_version::XRaySoundCommentVersion;
use crate::xray::xray_sound_parameters::XRaySoundParameters;

/// What the engine reads of an Ogg/Vorbis sound when it loads it, `CSoundRender_Source::LoadWave`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundFile {
  pub channels: u16,
  pub sample_rate: u32,
  /// Samples each channel plays, `ov_pcm_total`.
  pub samples_count: u64,
  pub metadata: SoundMetadata,
}

impl SoundFile {
  /// Read the Ogg/Vorbis headers and X-Ray metadata of a sound file.
  ///
  /// Successful reads guarantee Ogg/Vorbis headers libvorbis opens. Sounds without a recognized X-Ray comment use the
  /// engine's default source parameters. Use [`Self::read_strictly_from_path`] to fully decode the audio payload.
  pub fn read_from_path<P>(path: P) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    Self::read_from_path_with_strictness(path, false)
  }

  /// Read the headers and X-Ray metadata of a sound held in memory.
  pub fn read_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    Self::read_from(Cursor::new(bytes), false).map_err(Self::new_memory_error)
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
    Self::read_from(Cursor::new(bytes), true).map_err(Self::new_memory_error)
  }

  /// The sound held in `bytes` with `parameters` as its X-Ray comment, the version 3 block
  /// `CSoundManager::MakeGameSound` encodes a sound with. Its audio packets and every other comment are kept as they
  /// were; a first comment that is not an X-Ray block the engine reads stays behind the new one.
  ///
  /// # Errors
  ///
  /// Returns an error for parameters the engine refuses ([`XRaySoundParameters::validate`]), or bytes that are not a
  /// sound [`Self::read_from_bytes`] reads, or that carry a second Ogg stream.
  pub fn write_parameters(bytes: &[u8], parameters: &XRaySoundParameters) -> XrfResult<Vec<u8>> {
    let xray_comment: Vec<u8> = parameters.write()?;

    Self::rewrite_comment(bytes, xray_comment)
      .map_err(|error| XrfError::new_verify_error(format!("Failed to write sound parameters: {error}")))
  }

  /// Seconds the sound plays for, `CSoundRender_Source::length_sec`.
  pub fn get_duration(&self) -> Duration {
    let sample_rate: u64 = u64::from(self.sample_rate);
    let nanoseconds: u64 = (self.samples_count % sample_rate) * 1_000_000_000 / sample_rate;

    Duration::new(
      self.samples_count / sample_rate,
      u32::try_from(nanoseconds).expect("a remainder of a second is under a billion nanoseconds"),
    )
  }

  fn read_from_path_with_strictness<P>(path: P, is_strict: bool) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    let path: &Path = path.as_ref();

    File::open(path)
      .map_err(|error| format!("Could not open sound: {error}"))
      .and_then(|file| Self::read_from(file, is_strict))
      .map_err(|error| XrfError::new_verify_error(format!("Failed to read sound {}: {error}", format_path(path))))
  }

  /// Shared by the path and in-memory readers so the two cannot disagree about what a sound file is.
  fn read_from<R: Read + Seek>(source: R, is_strict: bool) -> Result<Self, String> {
    let mut reader: PacketReader<R> = PacketReader::new(source);
    let stream: VorbisStream = VorbisStream::read(&mut reader)?;
    let metadata: SoundMetadata = SoundMetadata::read(&stream.comment)?;

    if is_strict {
      stream.decode(&mut reader)?;
    }

    Ok(Self {
      channels: stream.identification.channels,
      sample_rate: stream.identification.sample_rate,
      samples_count: stream.read_samples_count(&mut reader.into_inner())?,
      metadata,
    })
  }

  fn rewrite_comment(bytes: &[u8], xray_comment: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut reader: PacketReader<Cursor<&[u8]>> = PacketReader::new(Cursor::new(bytes));
    let mut stream: VorbisStream = VorbisStream::read(&mut reader)?;

    stream
      .comment
      .set_first(xray_comment, |first| XRaySoundCommentVersion::read(first).is_some());

    stream.rewrite(&mut reader)
  }

  fn new_memory_error(error: String) -> XrfError {
    XrfError::new_verify_error(format!("Failed to read sound from memory: {error}"))
  }
}
