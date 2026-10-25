use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_job::{JobHandle, JobOutcome};
use xrf_texture::{BuildTextureOptions, BuildTextureProcessor, BuildTextureResult};
use xrf_utils::{error_to_string, to_portable_path_string};

use crate::edit::TextureBuildOmissionReport;

/// What a rebuilt texture came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBuildOutcome {
  /// Whether the texture was written or the run stopped before it started.
  pub outcome: JobOutcome,
  pub destination: String,
  /// Size of the source, which the descriptor's own width and height are refreshed from.
  pub width: u32,
  pub height: u32,
  /// Levels written, counting the base.
  pub mipmap_levels: u32,
  /// Recipe fields the descriptor asked for and the build did not carry out.
  pub omissions: Vec<TextureBuildOmissionReport>,
}

impl TextureBuildOutcome {
  /// Builds the texture `options` describe, unless the run was already asked to stop.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture cannot be encoded or written.
  pub fn build(job: &JobHandle, options: &BuildTextureOptions) -> TauriResult<Self> {
    // Read before the encode rather than after it. `BuildTextureProcessor::build` writes the file it encodes, so a
    // check on the far side would be reporting a stop for a run that had already published its result.
    if job.is_cancelled() {
      return Ok(Self {
        outcome: JobOutcome::Cancelled,
        destination: to_portable_path_string(&options.destination),
        width: 0,
        height: 0,
        mipmap_levels: 0,
        omissions: Vec::new(),
      });
    }

    let built: BuildTextureResult = BuildTextureProcessor::build(options).map_err(error_to_string)?;

    log::info!(
      "Built texture {}x{} with {} levels, {} recipe fields left out",
      built.width,
      built.height,
      built.mipmap_levels,
      built.omissions.len()
    );

    Ok(Self {
      outcome: JobOutcome::Completed,
      destination: to_portable_path_string(&built.destination),
      width: built.width,
      height: built.height,
      mipmap_levels: built.mipmap_levels,
      omissions: built
        .omissions
        .iter()
        .map(|omission| TextureBuildOmissionReport {
          field: omission.field().to_owned(),
          reason: omission.reason().to_owned(),
        })
        .collect(),
    })
  }
}
