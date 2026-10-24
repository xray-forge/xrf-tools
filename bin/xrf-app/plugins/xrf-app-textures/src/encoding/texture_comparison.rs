use std::path::Path;

use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionId;
use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsFile, DdsMipChain, DdsMipmaps, RgbaImage};
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_texture::TextureEncodingFormat;
use xrf_utils::{error_to_string, format_path};
use xrf_vfs::XrayRoots;

use crate::describe::{TextureFiles, TextureSource};
use crate::encoding::{
  TextureEncodingComparison, TextureEncodingCurrent, TextureEncodingReport, TexturesCompareRequest,
};
use crate::lease::TEXTURE_PHASE_WEIGH;

/// The encodes one comparison produced, kept until somebody saves one or asks for another texture, with the texture
/// they were made of: the only file a save may write them over.
pub struct TextureComparison {
  pub roots: XrayRoots,
  pub source: TextureSource,
  /// What to call it in a message, carried beside the source because a source is an address rather than a name.
  pub label: String,
  /// The texture's two files on disk, or none for one served out of an archive, which has nowhere to be written.
  pub files: Option<TextureFiles>,
  pub attempts: Vec<DdsEncodeAttempt>,
}

impl TextureComparison {
  /// Weighs every candidate against the texture `request` names, whose `bytes` were read from `files`: decoded once,
  /// reduced once over `mipmaps`, and encoded from those levels. Answers the comparison, the texture as it stands, and
  /// whether every candidate was reached.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are no texture, or a candidate cannot be encoded.
  pub fn measure(
    job: &JobHandle,
    request: &TexturesCompareRequest,
    bytes: &[u8],
    files: Option<TextureFiles>,
    mipmaps: DdsMipmaps,
  ) -> TauriResult<(Self, TextureEncodingCurrent, JobOutcome)> {
    let file: DdsFile = DdsFile::read_from_bytes(bytes).map_err(error_to_string)?;
    let base: RgbaImage = file.decode_rgba(0).map_err(error_to_string)?;
    let chain: DdsMipChain = DdsMipChain::build(&base, mipmaps).map_err(error_to_string)?;

    let weighing: JobScope = job.enter(TEXTURE_PHASE_WEIGH, Some(DdsEncodeCandidate::ALL.len() as u64));
    let mut attempts: Vec<DdsEncodeAttempt> = Vec::with_capacity(DdsEncodeCandidate::ALL.len());
    let mut outcome: JobOutcome = JobOutcome::Completed;

    for candidate in DdsEncodeCandidate::ALL {
      if job.is_cancelled() {
        outcome = JobOutcome::Cancelled;
        break;
      }

      job.set_detail(Some(candidate.label().to_owned()));
      attempts
        .push(DdsEncodeAttempt::measure(&chain, candidate, request.quality.to_quality()).map_err(error_to_string)?);
      weighing.advance();
    }

    job.set_detail(None);

    let comparison: Self = Self {
      roots: request.roots.clone(),
      source: request.source.clone(),
      label: request.source.to_label(),
      files,
      attempts,
    };

    Ok((comparison, TextureEncodingCurrent::of(&file.metadata()), outcome))
  }

  /// The encoded file for one candidate, or nothing when this comparison never weighed it.
  pub fn get(&self, format: TextureEncodingFormat) -> Option<&DdsFile> {
    let candidate: DdsEncodeCandidate = format.to_candidate();

    self
      .attempts
      .iter()
      .find(|attempt| attempt.candidate == candidate)
      .map(|attempt| &attempt.file)
  }

  /// The encoded file for one candidate.
  ///
  /// # Errors
  ///
  /// Returns an error naming the texture when this comparison did not finish encoding that format.
  pub fn require(&self, format: TextureEncodingFormat) -> TauriResult<&DdsFile> {
    self.get(format).ok_or_else(|| {
      format!(
        "The held comparison of '{}' does not carry that format; compare the formats again",
        self.label
      )
    })
  }

  /// The files a save of these encodes writes, given the descriptor the save names.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture has no file on disk, or when `descriptor` is not the one paired with it: a
  /// save must not write one texture's encodes beside another's descriptor.
  pub fn require_paired(&self, descriptor: &Path) -> TauriResult<&TextureFiles> {
    let files: &TextureFiles = self.files.as_ref().ok_or_else(|| {
      format!(
        "The held comparison of '{}' is of a texture served out of an archive, which has nowhere to be written",
        self.label
      )
    })?;

    if files.descriptor != descriptor {
      return Err(format!(
        "The held comparison is of '{}', whose descriptor is '{}', not '{}'",
        self.label,
        format_path(&files.descriptor),
        format_path(descriptor)
      ));
    }

    Ok(files)
  }

  /// What the comparison held as `session_id` says, which is everything about the attempts except their bytes.
  pub fn to_comparison(
    &self,
    session_id: SessionId,
    current: TextureEncodingCurrent,
    outcome: JobOutcome,
  ) -> TextureEncodingComparison {
    TextureEncodingComparison {
      session_id,
      source: self.source.clone(),
      roots: self.roots.clone(),
      outcome,
      reference: self.label.clone(),
      current,
      candidates: self.attempts.iter().map(TextureEncodingReport::of).collect(),
    }
  }
}
