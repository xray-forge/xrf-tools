use xrf_error::XrfResult;
use xrf_sound::SoundFile;

use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::project::sounds::sound_files_verification_result::GamedataSoundFilesVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

pub(crate) struct SoundFilesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
  sound_paths: &'a [String],
}

impl<'a> SoundFilesVerifier<'a> {
  pub(crate) fn new(
    project: &'a GamedataProject,
    options: &'a GamedataProjectVerifyOptions,
    sound_paths: &'a [String],
  ) -> Self {
    Self {
      options,
      project,
      sound_paths,
    }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataSoundFilesVerificationResult> {
    self.options.job.check_cancelled()?;

    let outcome: GamedataAssetSweepOutcome = GamedataAssetSweep::new(self.options)
      .run(self.sound_paths, |options, path| {
        self.verify_sound(options, path).into_iter().collect()
      })?;

    self.options.job.check_cancelled()?;

    Ok(GamedataSoundFilesVerificationResult {
      checked_sounds_count: outcome.checked_count,
      invalid_sounds_count: outcome.invalid_count,
      findings: outcome.findings,
    })
  }

  /// The finding decoding one sound raises, read through the VFS so an archived sound is decoded too.
  fn verify_sound(&self, options: &GamedataProjectVerifyOptions, path: &str) -> Option<Finding> {
    xrf_output::verbose!(options.output, "Verify sound: {path}");

    let error = self
      .project
      .read_bytes(path)
      .and_then(|bytes| {
        if options.is_strict {
          SoundFile::read_strictly_from_bytes(&bytes)
        } else {
          SoundFile::read_from_bytes(&bytes)
        }
      })
      .err()?;

    xrf_output::error!(options.output, "Sound is not valid: {path} - {error}");

    Some(GamedataFindingFactory::for_asset(
      GamedataVerificationRule::SoundsFiles,
      path,
      error.to_string(),
    ))
  }
}
