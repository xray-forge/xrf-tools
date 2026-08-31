use std::path::Path;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayLookupScope, XrayRoots, XrayScopedVfs, XrayVfs};

use crate::json::read::{parse_json, read_json};
use crate::language::TranslationLanguage;
use crate::project::verify::options::ProjectVerifyOptions;
use crate::project::verify::result::{ProjectVerifyLanguageSummary, ProjectVerifyResult};
use crate::source_file_name::{is_json_source, is_json_source_name};
use crate::types::TranslationJson;

/// Verify every translation source the roots expose.
///
/// Reads through `xrf-vfs`, so a source tree layered over an installation verifies as readily as a
/// loose one, and the sources it judges are the ones the engine would actually load rather than the
/// ones a directory walk happens to find first.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted or the prefix is not a logical path. An
/// unreadable source is fatal, as it has always been here: a verdict over a file this could not parse
/// would be a verdict about nothing.
pub fn verify_roots(
  roots: &XrayRoots,
  prefix: Option<&str>,
  options: &ProjectVerifyOptions,
) -> XrfResult<ProjectVerifyResult> {
  verify_roots_in(&roots.open()?, prefix, options)
}

/// Verify over roots somebody else mounted.
///
/// # Errors
///
/// The same as [`verify_roots`], minus mounting.
pub fn verify_roots_in(
  vfs: &XrayVfs,
  prefix: Option<&str>,
  options: &ProjectVerifyOptions,
) -> XrfResult<ProjectVerifyResult> {
  let scope: XrayLookupScope = XrayLookupScope::all().with_optional_prefix(prefix)?;
  let scoped: XrayScopedVfs = vfs.scoped(&scope);

  let started_at: Instant = Instant::now();
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  let mut assets: Vec<XrayAsset> = scoped
    .list_entries()
    .into_iter()
    .filter(|asset| is_json_source_name(asset.get_logical_path().file_name()))
    .collect();

  // Sorted because mount order is not name order, and a verdict is only comparable across runs and
  // machines if it depends on neither.
  assets.sort_by(|left, right| left.get_logical_path().as_str().cmp(right.get_logical_path().as_str()));

  // Said once the sources are known, because the count is the useful half: the caller already named
  // what it pointed at, and repeating a prefix it chose tells it nothing it did not just type.
  xrf_output::info!(options.output, "Verifying {} translation source(s)", assets.len());

  let verifying: xrf_job::JobScope = options.job.enter(
    crate::project::job_phases::TRANSLATION_PHASE_VERIFY,
    Some(assets.len() as u64),
  );

  for asset in &assets {
    // A stopped verification reports what it had judged and says so. Its silence about the rest is not a verdict.
    if options.job.is_cancelled() {
      result.outcome = xrf_job::JobOutcome::Cancelled;

      break;
    }

    let logical_path: &str = asset.get_logical_path().as_str();

    options.job.set_detail(Some(logical_path.to_owned()));

    let parsed: TranslationJson = scoped
      .read_asset_bytes(asset)
      .and_then(|data| parse_json(logical_path, &data))?;

    result.merge(verify_parsed(logical_path, &parsed, options));
    verifying.advance();
  }

  result.duration = started_at.elapsed();

  log::info!(
    "Verified {} translation source(s) in {}",
    assets.len(),
    xrf_utils::format_duration(result.duration)
  );

  Ok(result)
}

/// Verify one source, skipping anything that is not a multi-language JSON.
///
/// The path-taking convenience, for a caller holding one file and no mounted roots - the same shape
/// `read_json` and `read_decoded` keep, and the reason `--path <one file>` still works. A VFS mounts a
/// directory, never a file.
///
/// # Errors
///
/// Returns a parsing error for an unreadable source.
pub fn verify_file<P: AsRef<Path>>(path: &P, options: &ProjectVerifyOptions) -> XrfResult<ProjectVerifyResult> {
  // Through the shared parser rather than comparing an extension, so this recognises the same names
  // the reader does. An exact compare skipped `ST_A.JSON` with only an info line, while the editor
  // opened it — the VFS lower-cases logical paths and the host walk does not.
  if !is_json_source(path.as_ref()) {
    log::info!("Skip file {}", format_path(path.as_ref()));
    xrf_output::info!(options.output, "Skip file {}", format_path(path.as_ref()));

    return Ok(ProjectVerifyResult::new());
  }

  let path_display: String = format_path(path.as_ref()).to_string();

  log::info!("Verifying JSON file {}", path_display);

  let started_at: Instant = Instant::now();
  let parsed: TranslationJson = read_json(path.as_ref())?;
  let mut result: ProjectVerifyResult = verify_parsed(&path_display, &parsed, options);

  result.duration = started_at.elapsed();

  Ok(result)
}

/// Record what each requested language is missing from one already-parsed source.
fn verify_parsed(subject: &str, parsed: &TranslationJson, options: &ProjectVerifyOptions) -> ProjectVerifyResult {
  let mut result: ProjectVerifyResult = ProjectVerifyResult::new();

  let languages: Vec<String> = if options.language == TranslationLanguage::All {
    TranslationLanguage::get_all_strings()
  } else {
    vec![options.language.to_string()]
  };

  for language in languages {
    let mut missing: u32 = 0;

    for (key, entry) in parsed {
      // A present-but-null entry counts as missing: it is a placeholder waiting for a translator.
      if entry.get(&language).is_none_or(|translation| translation.is_none()) {
        missing += 1;

        xrf_output::error!(
          options.output,
          "Translation key missing: {} {} in {}",
          key,
          language,
          subject
        );

        result.record_missing_translation(Path::new(subject), key, &language, options.is_detailed);
      }
    }

    result.record_language_summary(ProjectVerifyLanguageSummary {
      file: subject.to_owned(),
      language,
      checked: parsed.len() as u32,
      missing,
    });
  }

  result.checked_translations_count = parsed.len() as u32;

  result
}
