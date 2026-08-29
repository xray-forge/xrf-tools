use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_utils::{encode_string_to_bytes, format_path};
use xrf_vfs::{XrayAsset, XrayLookupScope, XrayRoots, XrayScopedVfs, XrayVfs};

use crate::json::read::{parse_json, read_json};
use crate::language::TranslationLanguage;
use crate::project::build::compile::compile_by_language;
use crate::project::build::options::ProjectBuildOptions;
use crate::project::build::result::ProjectBuildResult;
use crate::project::build::targets::{ensure_output_outside_roots, prepare_target_file, validate_targets};
use crate::source_file_name::{is_json_source, is_json_source_name};
use crate::types::TranslationJson;

/// Build every translation source the roots expose.
///
/// Reads through `xrf-vfs`, so a source tree layered over an installation compiles the sources the
/// engine would actually load. Writes to the host, because a string table is a file and there is
/// nowhere to put one inside a volume - which is why the output is a plain path rather than roots.
///
/// Targets are validated before anything is written, so a build that would have two sources fighting
/// over one output file fails having produced nothing.
///
/// # Errors
///
/// Returns an invalid error for colliding targets or for output inside a source root, and whatever
/// building an individual source returns.
pub fn build_roots(
  roots: &XrayRoots,
  prefix: Option<&str>,
  options: &ProjectBuildOptions,
) -> XrfResult<ProjectBuildResult> {
  ensure_output_outside_roots(roots, &options.output_dir)?;

  build_roots_in(&roots.open()?, prefix, options)
}

/// Build over roots somebody else mounted.
///
/// # Errors
///
/// The same as [`build_roots`], minus mounting and the output guard the caller has already applied.
pub fn build_roots_in(
  vfs: &XrayVfs,
  prefix: Option<&str>,
  options: &ProjectBuildOptions,
) -> XrfResult<ProjectBuildResult> {
  let scope: XrayLookupScope = XrayLookupScope::all().with_optional_prefix(prefix)?;
  let scoped: XrayScopedVfs = vfs.scoped(&scope);

  let started_at: Instant = Instant::now();
  let mut result: ProjectBuildResult = ProjectBuildResult::new();

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

  xrf_output::info!(options.output, "Building {} translation source(s)", assets.len());

  for asset in &assets {
    let logical_path: &str = asset.get_logical_path().as_str();
    let parsed: TranslationJson = scoped
      .read_asset_bytes(asset)
      .and_then(|data| parse_json(logical_path, &data))?;

    result.sources += 1;

    build_parsed(logical_path, &parsed, options, &mut result)?;
  }

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
/// The path-taking convenience the readers keep, and the reason `--path <one source>` still works: a
/// VFS mounts a directory, never a file.
///
/// # Errors
///
/// Returns whatever building the JSON source returns.
pub fn build_file<P: AsRef<Path>>(path: &P, options: &ProjectBuildOptions) -> XrfResult<ProjectBuildResult> {
  let started_at: Instant = Instant::now();
  let mut result: ProjectBuildResult = ProjectBuildResult::new();

  // Through the shared parser rather than comparing an extension; see `verify_file` for why.
  if is_json_source(path.as_ref()) {
    let parsed: TranslationJson = read_json(path.as_ref())?;

    result.sources += 1;

    build_parsed(&format_path(path.as_ref()).to_string(), &parsed, options, &mut result)?;
  } else {
    log::info!("Skip file {}", format_path(path.as_ref()));
    xrf_output::info!(options.output, "Skip file {}", format_path(path.as_ref()));
  }

  result.duration = started_at.elapsed();

  Ok(result)
}

/// Compile one parsed source into one string table per language the run asked for.
fn build_parsed(
  subject: &str,
  parsed: &TranslationJson,
  options: &ProjectBuildOptions,
  result: &mut ProjectBuildResult,
) -> XrfResult {
  xrf_output::verbose!(options.output, "Building translations {subject}");

  if options.language == TranslationLanguage::All {
    for language in TranslationLanguage::get_all() {
      build_language(subject, parsed, &language, options, result)?;
    }
  } else {
    build_language(subject, parsed, &options.language, options, result)?;
  }

  Ok(())
}

fn build_language(
  subject: &str,
  source: &TranslationJson,
  language: &TranslationLanguage,
  options: &ProjectBuildOptions,
  result: &mut ProjectBuildResult,
) -> XrfResult {
  let compiled: String = compile_by_language(Path::new(subject), source, language, options)?;
  let data: Vec<u8> = encode_string_to_bytes(&compiled, language.new_language_encoder())?;
  let target: PathBuf = crate::project::build::targets::target_path(subject, &options.output_dir, language)?;

  prepare_target_file(&target, options)?.write_all(&data)?;

  result.record_built_file(&language.to_string(), source.len() as u32);

  Ok(())
}
