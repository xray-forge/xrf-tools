use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_job::{JobHandle, JobOutcome};
use xrf_texture::{BuildTextureOptions, BuildTextureProcessor, BuildTextureResult, GenerateBumpResult};
use xrf_utils::{error_to_string, to_portable_path_string};

use crate::edit::TextureBuildOmissionReport;

/// What a rebuilt texture came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBuildOutcome {
  /// Whether the texture was written or the run stopped before it started.
  pub outcome: JobOutcome,
  pub destination: String,
  /// The `#` half a bump descriptor writes beside the texture, the compression error and the height; absent for any
  /// other descriptor.
  pub companion: Option<String>,
  /// Mean gloss of a bump descriptor's pair, in `0..=1`; absent for any other descriptor.
  pub gloss_power: Option<f32>,
  /// Whether that gloss is too dark for the surface to show a specular response worth having.
  pub is_gloss_too_dark: bool,
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
        companion: None,
        gloss_power: None,
        is_gloss_too_dark: false,
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
      companion: built.bump.as_ref().map(|pair| to_portable_path_string(&pair.companion)),
      gloss_power: built.bump.as_ref().map(|pair| pair.gloss_power),
      is_gloss_too_dark: built.bump.as_ref().is_some_and(GenerateBumpResult::is_gloss_too_dark),
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
