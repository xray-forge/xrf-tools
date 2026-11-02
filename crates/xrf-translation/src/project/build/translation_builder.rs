use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostDestination, HostRelativePath, format_path};
use xrf_vfs::{XrayAsset, XrayRoots, XraySearch};

use crate::json::translation_json_reader::TranslationJsonReader;
use crate::json::translation_source_name::TranslationSourceName;
use crate::project::build::compiled::translation_table_compiler::TranslationTableCompiler;
use crate::project::build::translation_build_options::TranslationBuildOptions;
use crate::project::build::translation_build_result::TranslationBuildResult;
use crate::project::build::translation_build_targets::TranslationBuildTargets;
use crate::text::translation_json::TranslationJson;

/// Compiles translation sources into the per-language string tables the game loads.
pub struct TranslationBuilder;

impl TranslationBuilder {
  /// Phase a build reports while it compiles sources into string tables.
  pub const PHASE: &str = "build";

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
    TranslationBuildTargets::ensure_output_outside_roots(roots, &options.output_dir)?;

    Self::build_roots_in(&roots.to_search_plan()?.open()?, prefix, options)
  }

  /// Build over roots somebody else mounted.
  ///
  /// # Errors
  ///
  /// The same as [`Self::build_roots`], minus mounting and the output guard the caller has already applied.
  pub fn build_roots_in(
    search: &XraySearch,
    prefix: Option<&str>,
    options: &TranslationBuildOptions,
  ) -> XrfResult<TranslationBuildResult> {
    let sources: XraySearch = search.clone().with_optional_prefix(prefix)?;

    let started_at: Instant = Instant::now();
    let mut result: TranslationBuildResult = TranslationBuildResult::new();

    // Ordered by engine identity rather than mount order, so a build is comparable across runs and machines.
    let assets: Vec<XrayAsset> = sources
      .list_files()
      .into_iter()
      .filter(|asset| TranslationSourceName::is_name(asset.get_logical_path().file_name()))
      .collect();

    let names: Vec<String> = assets
      .iter()
      .map(|asset| asset.get_logical_path().as_str().to_owned())
      .collect();

    TranslationBuildTargets::validate(&names, options)?;

    let destination: HostDestination = HostDestination::new(&options.output_dir, options.link_policy);

    xrf_output::info!(options.output, "Building {} translation source(s)", assets.len());

    let building: xrf_job::JobScope = options.job.enter(Self::PHASE, Some(assets.len() as u64));

    for asset in &assets {
      // Between sources: each string table is written whole, so stopping here leaves the ones already built valid and
      // the rest simply absent.
      if options.job.is_cancelled() {
        result.outcome = xrf_job::JobOutcome::Cancelled;

        break;
      }

      let logical_path: &str = asset.get_logical_path().as_str();

      options.job.set_detail(Some(logical_path.to_owned()));

      let parsed: TranslationJson = sources
        .read_asset(asset)
        .and_then(|data| TranslationJsonReader::parse(logical_path, &data))?;

      result.sources += 1;

      Self::build_parsed(
        &format_path(Path::new(logical_path)).to_string(),
        logical_path,
        &parsed,
        &destination,
        options,
        &mut result,
      )?;

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

    // Through the shared parser rather than comparing an extension; see `TranslationVerifier::verify_file` for why.
    if TranslationSourceName::is_path(path.as_ref()) {
      // The file name alone names the target: the rest of a host path says where the source was, not where its
      // string table goes, and a parent component in it would carry the write out of the output directory.
      let name: &str = path.as_ref().file_name().and_then(OsStr::to_str).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Translation source '{}' has no file name to build to",
          format_path(path.as_ref())
        ))
      })?;

      TranslationBuildTargets::validate(&[name.to_owned()], options)?;

      let parsed: TranslationJson = TranslationJsonReader::read(path.as_ref())?;
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

    for language in TranslationBuildTargets::languages_for(target_name, options.language) {
      let data: Vec<u8> = TranslationTableCompiler::compile(subject, parsed, language, options.is_sorted)?;
      let target: HostRelativePath = TranslationBuildTargets::target_path(target_name, language)?;

      xrf_output::verbose!(
        options.output,
        "Writing file {}",
        format_path(&destination.get_root().join(&target))
      );

      destination.create_file(&target)?.write_all(&data)?;

      result.record_built_file(&language.to_string(), parsed.len() as u32);
    }

    Ok(())
  }
}
