use std::time::Instant;

use xrf_error::XrfResult;
use xrf_vfs::XrayAssetType as AssetType;

use crate::project::sounds::sound_files_verifier::SoundFilesVerifier;
use crate::project::sounds::sound_references_verifier::SoundReferencesVerifier;
use crate::project::sounds::sounds_verification_result::GamedataSoundsVerificationResult;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions};

pub(crate) struct SoundsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> SoundsVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataSoundsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify sounds:");

    let started_at: Instant = Instant::now();

    // Enumerated through the VFS, so an installation's archived sounds count too. The index only ever sees one loose
    // directory.
    let sound_paths: Vec<String> = self
      .project
      .vfs()
      .scoped(&self.project.scope)
      .list_entries_of_type(AssetType::Ogg)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let sound_files = SoundFilesVerifier::new(self.project, self.options, &sound_paths).verify()?;
    let sound_references = SoundReferencesVerifier::new(self.project, self.options, &sound_paths).verify()?;

    let result: GamedataSoundsVerificationResult =
      GamedataSoundsVerificationResult::new(started_at.elapsed(), sound_files, sound_references);

    xrf_output::info!(
      self.options.output,
      "Verified gamedata sounds in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_failure_message()
    );

    self.options.job.check_cancelled()?;

    Ok(result)
  }
}
