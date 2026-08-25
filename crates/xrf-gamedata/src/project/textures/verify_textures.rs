use std::path::Path;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xrf_db::{ThmFile, XRayByteOrder};
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::textures::verify_textures_result::GamedataTexturesVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  pub fn verify_textures(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataTexturesVerificationResult> {
    xrf_output::heading!(options.output, "Verify textures:");

    let started_at: Instant = Instant::now();

    let texture_paths: Vec<String> = self
      .entries_of_type(AssetType::Dds)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let checked_textures_count: u32 = u32::try_from(texture_paths.len())
      .map_err(|_| XrfError::new_verify_error("Texture count exceeds the supported result range"))?;

    // Textures are read in parallel, so each one logs into its listed position and the sequence
    // releases them in path order rather than in the order the workers finished.
    let sequence: OutputSequence = OutputSequence::new(&options.output, texture_paths.len());

    let mut findings: Vec<Finding> = texture_paths
      .par_iter()
      .enumerate()
      .filter_map(|(index, relative_path)| {
        let slot: OutputSlot = sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();

        xrf_output::verbose!(output, "Verify texture: {relative_path}");

        let path: &String = relative_path;

        match self.verify_texture_by_path(path) {
          Ok(true) => None,
          Ok(false) => {
            xrf_output::info!(output, "Texture is not valid: {}", path);

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::TexturesValidation,
              path,
              "Texture uses an unsupported format",
            ))
          }
          Err(error) => {
            xrf_output::info!(output, "Texture verification failed: {} - {}", path, error);

            Some(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::TexturesRead,
              path,
              error.to_string(),
            ))
          }
        }
      })
      .collect();

    let invalid_textures_count: u32 = u32::try_from(findings.len())
      .map_err(|_| XrfError::new_verify_error("Invalid texture count exceeds the supported result range"))?;

    let (bump_findings, checked_bumps_count) = self.verify_texture_bumps(options)?;
    let unresolved_bumps_count: u32 = u32::try_from(bump_findings.len())
      .map_err(|_| XrfError::new_verify_error("Unresolved bump count exceeds the supported result range"))?;

    findings.extend(bump_findings);
    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_and_message);

    let duration: Duration = started_at.elapsed();

    xrf_output::info!(
      options.output,
      "Verified gamedata textures in {}, {}/{} valid, {}/{} declared bumps resolved",
      xrf_utils::format_duration(duration),
      checked_textures_count - invalid_textures_count,
      checked_textures_count,
      checked_bumps_count - unresolved_bumps_count,
      checked_bumps_count
    );

    Ok(GamedataTexturesVerificationResult {
      duration,
      findings,
      invalid_textures_count,
      checked_textures_count,
      checked_bumps_count,
      unresolved_bumps_count,
    })
  }

  /// Check that every bump a texture descriptor asks for actually exists.
  ///
  /// `CTextureDescrMngr::LoadTHM` takes the thm bump name verbatim, with no `_bump` naming
  /// convention behind it. A name that resolves to nothing does not turn bump mapping off, because
  /// `bump_exist` only tests that the name is non-empty: the renderer still takes the `_bump`
  /// shader path and the loader substitutes `ed\ed_dummy_bump`, so the surface is flat anyway and
  /// the log fills with `! Fallback to default bump map`. Importing a texture under a path
  /// different from its source is the usual way to produce one, because the copied descriptor
  /// keeps pointing into the source layout.
  ///
  /// Returns the findings and how many descriptors declared a bump at all.
  fn verify_texture_bumps(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<(Vec<Finding>, u32)> {
    let descriptor_paths: Vec<String> = self
      .entries_of_type(AssetType::Thm)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    // Descriptors are read in parallel; the sequence releases what each one says in path order.
    let descriptor_sequence: OutputSequence = OutputSequence::new(&options.output, descriptor_paths.len());

    let declarations: Vec<(String, String)> = descriptor_paths
      .par_iter()
      .enumerate()
      .filter_map(|(index, relative_path)| {
        let slot: OutputSlot = descriptor_sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();

        match self.read_parsed(AssetType::Thm, relative_path, |chunk| {
          ThmFile::read_from_chunk::<XRayByteOrder, _>(chunk)
        }) {
          Ok(descriptor) => descriptor
            .used_bump_name()
            .map(|bump_name| (relative_path.clone(), bump_name.to_owned())),
          Err(error) => {
            // A descriptor that cannot be parsed is reported by its own texture, not silently
            // treated as declaring no bump.
            xrf_output::verbose!(output, "Texture descriptor is not readable: {relative_path} - {error}");

            None
          }
        }
      })
      .collect();

    let checked_bumps_count: u32 = u32::try_from(declarations.len())
      .map_err(|_| XrfError::new_verify_error("Declared bump count exceeds the supported result range"))?;

    let declaration_sequence: OutputSequence = OutputSequence::new(&options.output, declarations.len());

    let findings: Vec<Finding> = declarations
      .par_iter()
      .enumerate()
      .filter_map(|(index, (relative_path, bump_name))| {
        let slot: OutputSlot = declaration_sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();

        if self.dds_texture(bump_name).ok().flatten().is_some() {
          return None;
        }

        xrf_output::info!(
          output,
          "Texture descriptor declares missing bump: {relative_path} -> {bump_name}"
        );

        Some(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::TexturesBump,
          Path::new(relative_path),
          format!("Texture descriptor declares bump '{bump_name}' that is not in gamedata"),
        ))
      })
      .collect();

    Ok((findings, checked_bumps_count))
  }

  /// Whether one texture reads as an X-Ray compatible DDS, addressed by its logical path.
  ///
  /// Reads through the VFS, so an archived texture is inspected rather than skipped.
  pub(crate) fn verify_texture_by_path(&self, logical_path: &str) -> XrfResult<bool> {
    Ok(DdsFile::read_from_bytes(&self.read_bytes(logical_path)?)?.is_xray_compatible())
  }
}
