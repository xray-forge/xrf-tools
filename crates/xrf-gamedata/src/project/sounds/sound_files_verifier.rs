use std::path::Path;

use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_sound::SoundFile;

use crate::GamedataFindingFactory;
use crate::project::sounds::sound_files_verification_result::GamedataSoundFilesVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

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
    let checked_sounds_count: u32 = u32::try_from(self.sound_paths.len())
      .map_err(|_| XrfError::new_verify_error("Sound count exceeds the supported result range"))?;

    let mut findings: Vec<Finding> = self
      .sound_paths
      .par_iter()
      .filter_map(|relative_path| {
        xrf_output::verbose!(self.options.output, "Verify sound: {relative_path}");

        // Read through the VFS, so an archived sound is decoded rather than reported missing.
        let bytes: Vec<u8> = match self.project.read_bytes(relative_path) {
          Ok(bytes) => bytes,
          Err(_) => {
            return Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::SoundsFiles,
              Path::new(relative_path),
              "Sound path was not found in gamedata roots",
            ));
          }
        };

        let sound: XrfResult<SoundFile> = if self.options.is_strict {
          SoundFile::read_strictly_from_bytes(&bytes)
        } else {
          SoundFile::read_from_bytes(&bytes)
        };

        sound.err().map(|error| {
          xrf_output::error!(self.options.output, "Sound is not valid: {relative_path} - {error}");

          GamedataFindingFactory::for_asset(
            GamedataVerificationRule::SoundsFiles,
            Path::new(relative_path),
            error.to_string(),
          )
        })
      })
      .collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    let invalid_sounds_count: u32 = u32::try_from(findings.len())
      .map_err(|_| XrfError::new_verify_error("Invalid sound count exceeds the supported result range"))?;

    Ok(GamedataSoundFilesVerificationResult {
      checked_sounds_count,
      invalid_sounds_count,
      findings,
    })
  }
}
