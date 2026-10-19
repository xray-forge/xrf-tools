use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostDestination, HostRelativePath, encode_string_to_bytes, format_path};
use xrf_vfs::{XrayAsset, XrayLookupScope, XrayRoots, XrayScopedVfs, XrayVfs};

use crate::json::read::{parse_json, read_json};
use crate::language::TranslationLanguage;
use crate::project::build::compile::compile_by_language;
use crate::project::build::targets::{ensure_output_outside_roots, target_path, validate_targets};
use crate::project::build::translation_build_options::TranslationBuildOptions;
use crate::project::build::translation_build_result::TranslationBuildResult;
use crate::source_file_name::{is_json_source, is_json_source_name};
use crate::types::TranslationJson;

/// Compiles translation sources into the per-language string tables the game loads.
pub struct TranslationBuilder;

impl TranslationBuilder {
  /// Build every translation source the roots expose.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for colliding targets or for output inside a source root, and whatever
  /// building an individual source returns.
  pub fn build_roots(
    roots: &XrayRoots,
    prefix: Option<&str>,
    options: &TranslationBuildOptions,
  ) -> XrfResult<TranslationBuildResult> {
    ensure_output_outside_roots(roots, &options.output_dir)?;

    Self::build_roots_in(&roots.open()?, prefix, options)
  }

  /// Build over roots somebody else mounted.
  ///
  /// # Errors
  ///
  /// The same as [`Self::build_roots`], minus mounting and the output guard the caller has already applied.
  pub fn build_roots_in(
    vfs: &XrayVfs,
    prefix: Option<&str>,
    options: &TranslationBuildOptions,
  ) -> XrfResult<TranslationBuildResult> {
    let scope: XrayLookupScope = XrayLookupScope::all().with_optional_prefix(prefix)?;
    let scoped: XrayScopedVfs = vfs.scoped(&scope);

    let started_at: Instant = Instant::now();
    let mut result: TranslationBuildResult = TranslationBuildResult::new();

    let mut assets: Vec<XrayAsset> = scoped
      .list_entries()
      .into_iter()
      .filter(|asset| is_json_source_name(asset.get_logical_path().file_name()))
      .collect();

    // Sorted because mount order is not name order, and a build is only comparable across runs and
    // machines if it depends on neither.
    assets.sort_by(|left, right| left.get_logical_path().as_str().cmp(right.get_logical_path().as_str()));

    let names: Vec<String> = assets
      .iter()
      .map(|asset| asset.get_logical_path().as_str().to_owned())
      .collect();

    validate_targets(&names, options)?;

    let destination: HostDestination = HostDestination::new(&options.output_dir, options.link_policy);

    xrf_output::info!(options.output, "Building {} translation source(s)", assets.len());

    let building: xrf_job::JobScope = options.job.enter(
      crate::project::job_phases::TRANSLATION_PHASE_BUILD,
      Some(assets.len() as u64),
    );

    for asset in &assets {
      // Between sources: each string table is written whole, so stopping here leaves the ones already built valid and
      // the rest simply absent.
      if options.job.is_cancelled() {
        result.outcome = xrf_job::JobOutcome::Cancelled;

        break;
      }

      let logical_path: &str = asset.get_logical_path().as_str();

      options.job.set_detail(Some(logical_path.to_owned()));

      let parsed: TranslationJson = scoped
        .read_asset_bytes(asset)
        .and_then(|data| parse_json(logical_path, &data))?;

      result.sources += 1;

      Self::build_parsed(logical_path, logical_path, &parsed, &destination, options, &mut result)?;

      building.advance();
    }

    options.job.set_detail(None);

    result.duration = started_at.elapsed();

    log::info!(
      "Built {} string table(s) from {} source(s) in {}",
      result.files,
      result.sources,
      xrf_utils::format_duration(result.duration)
    );

    Ok(result)
  }

  /// Build one source off disk, for a caller holding one file and no mounted roots.
  ///
  /// # Errors
  ///
  /// Returns whatever building the JSON source returns.
  pub fn build_file<P: AsRef<Path>>(path: &P, options: &TranslationBuildOptions) -> XrfResult<TranslationBuildResult> {
    let started_at: Instant = Instant::now();
    let mut result: TranslationBuildResult = TranslationBuildResult::new();

    // Through the shared parser rather than comparing an extension; see `verify_file` for why.
    if is_json_source(path.as_ref()) {
      // The file name alone names the target: the rest of a host path says where the source was, not where its
      // string table goes, and a parent component in it would carry the write out of the output directory.
      let name: &str = path.as_ref().file_name().and_then(OsStr::to_str).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Translation source '{}' has no file name to build to",
          format_path(path.as_ref())
        ))
      })?;

      validate_targets(&[name.to_owned()], options)?;

      let parsed: TranslationJson = read_json(path.as_ref())?;
      let destination: HostDestination = HostDestination::new(&options.output_dir, options.link_policy);

      result.sources += 1;

      Self::build_parsed(
        &format_path(path.as_ref()).to_string(),
        name,
        &parsed,
        &destination,
        options,
        &mut result,
      )?;
    } else {
      log::info!("Skip file {}", format_path(path.as_ref()));
      xrf_output::info!(options.output, "Skip file {}", format_path(path.as_ref()));
    }

    result.duration = started_at.elapsed();

    Ok(result)
  }

  /// Compile one parsed source into one string table per language the run asked for.
  ///
  /// `subject` names the source in messages; `target_name` is the name its string tables are written under.
  fn build_parsed(
    subject: &str,
    target_name: &str,
    parsed: &TranslationJson,
    destination: &HostDestination,
    options: &TranslationBuildOptions,
    result: &mut TranslationBuildResult,
  ) -> XrfResult {
    xrf_output::verbose!(options.output, "Building translations {subject}");

    destination.create_root()?;

    if options.language == TranslationLanguage::All {
      for language in TranslationLanguage::get_all() {
        Self::build_language(subject, target_name, parsed, &language, destination, options, result)?;
      }
    } else {
      Self::build_language(
        subject,
        target_name,
        parsed,
        &options.language,
        destination,
        options,
        result,
      )?;
    }

    Ok(())
  }

  fn build_language(
    subject: &str,
    target_name: &str,
    source: &TranslationJson,
    language: &TranslationLanguage,
    destination: &HostDestination,
    options: &TranslationBuildOptions,
    result: &mut TranslationBuildResult,
  ) -> XrfResult {
    let compiled: String = compile_by_language(Path::new(subject), source, language, options)?;
    let data: Vec<u8> = encode_string_to_bytes(&compiled, language.new_language_encoder())?;
    let target: HostRelativePath = target_path(target_name, language)?;

    xrf_output::verbose!(
      options.output,
      "Writing file {}",
      format_path(&destination.get_root().join(&target))
    );

    destination.create_file(&target)?.write_all(&data)?;

    result.record_built_file(&language.to_string(), source.len() as u32);

    Ok(())
  }
}
