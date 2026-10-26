use std::path::Path;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayRoots, XraySearch};

use crate::json::translation_json_reader::TranslationJsonReader;
use crate::json::translation_source_name::TranslationSourceName;
use crate::language::translation_language::TranslationLanguage;
use crate::project::build::compiled::translation_table_compiler::TranslationTableCompiler;
use crate::project::build::translation_build_targets::TranslationBuildTargets;
use crate::project::describe::translation_finding::TranslationFinding;
use crate::project::verify::translation_verify_language_summary::TranslationVerifyLanguageSummary;
use crate::project::verify::translation_verify_options::TranslationVerifyOptions;
use crate::project::verify::translation_verify_result::TranslationVerifyResult;
use crate::text::translation_json::TranslationJson;

/// Judges translation sources: every id a requested language has no text for, and everything the game would refuse or
/// mangle in the tables a build would write from them.
pub struct TranslationVerifier;

impl TranslationVerifier {
  /// Phase a verification reports while it checks sources.
  pub const PHASE: &str = "verify";

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
    options: &TranslationVerifyOptions,
  ) -> XrfResult<TranslationVerifyResult> {
    Self::verify_roots_in(&roots.to_search_plan()?.open()?, prefix, options)
  }

  /// Verify over roots somebody else mounted.
  ///
  /// # Errors
  ///
  /// The same as [`Self::verify_roots`], minus mounting.
  pub fn verify_roots_in(
    search: &XraySearch,
    prefix: Option<&str>,
    options: &TranslationVerifyOptions,
  ) -> XrfResult<TranslationVerifyResult> {
    let sources: XraySearch = search.clone().with_optional_prefix(prefix)?;

    let started_at: Instant = Instant::now();
    let mut result: TranslationVerifyResult = TranslationVerifyResult::new();

    // Ordered by engine identity rather than mount order, so a verdict is comparable across runs and machines.
    let assets: Vec<XrayAsset> = sources
      .list_files()
      .into_iter()
      .filter(|asset| TranslationSourceName::is_name(asset.get_logical_path().file_name()))
      .collect();

    // Said once the sources are known, because the count is the useful half: the caller already named
    // what it pointed at, and repeating a prefix it chose tells it nothing it did not just type.
    xrf_output::info!(options.output, "Verifying {} translation source(s)", assets.len());

    let verifying: xrf_job::JobScope = options.job.enter(Self::PHASE, Some(assets.len() as u64));
    // Each source's ids, which is all the checks across sources need of it.
    let mut defined: Vec<(String, Vec<String>)> = Vec::new();

    for asset in &assets {
      // A stopped verification reports what it had judged and says so. Its silence about the rest is not a verdict.
      if options.job.is_cancelled() {
        result.outcome = xrf_job::JobOutcome::Cancelled;

        break;
      }

      let logical_path: &str = asset.get_logical_path().as_str();

      options.job.set_detail(Some(logical_path.to_owned()));

      let parsed: TranslationJson = sources
        .read_asset(asset)
        .and_then(|data| TranslationJsonReader::parse(logical_path, &data))?;

      result.merge(Self::verify_parsed(logical_path, &parsed, options)?);
      defined.push((logical_path.to_owned(), parsed.into_keys().collect()));
      verifying.advance();
    }

    options.job.set_detail(None);

    result
      .problems
      .extend(Self::find_cross_source_problems(&defined, options.language)?);
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
  /// [`TranslationJsonReader::read`] keeps, and the reason `--path <one file>` still works. A VFS mounts a directory,
  /// never a file.
  ///
  /// # Errors
  ///
  /// Returns a parsing error for an unreadable source.
  pub fn verify_file<P: AsRef<Path>>(
    path: &P,
    options: &TranslationVerifyOptions,
  ) -> XrfResult<TranslationVerifyResult> {
    // Through the shared parser rather than comparing an extension, so this recognises the same names
    // the reader does. An exact compare skipped `ST_A.JSON` with only an info line, while the editor
    // opened it — the VFS lower-cases logical paths and the host walk does not.
    if !TranslationSourceName::is_path(path.as_ref()) {
      log::info!("Skip file {}", format_path(path.as_ref()));
      xrf_output::info!(options.output, "Skip file {}", format_path(path.as_ref()));

      return Ok(TranslationVerifyResult::new());
    }

    let path_display: String = format_path(path.as_ref()).to_string();

    log::info!("Verifying JSON file {}", path_display);

    let started_at: Instant = Instant::now();
    let parsed: TranslationJson = TranslationJsonReader::read(path.as_ref())?;
    let mut result: TranslationVerifyResult = Self::verify_parsed(&path_display, &parsed, options)?;

    result.duration = started_at.elapsed();

    Ok(result)
  }

  /// Record what each requested language is missing from one already-parsed source, and what the game would refuse or
  /// mangle in the table it builds to.
  fn verify_parsed(
    subject: &str,
    parsed: &TranslationJson,
    options: &TranslationVerifyOptions,
  ) -> XrfResult<TranslationVerifyResult> {
    let mut result: TranslationVerifyResult = TranslationVerifyResult::new();

    for language in options.language.expand() {
      let code: String = language.to_string();
      let mut missing: u32 = 0;

      for (key, entry) in parsed {
        // A present-but-null entry counts as missing: it is a placeholder waiting for a translator.
        if entry.get(&code).is_none_or(|translation| translation.is_none()) {
          missing += 1;

          xrf_output::error!(
            options.output,
            "Translation key missing: {} {} in {}",
            key,
            code,
            subject
          );

          result.record_missing_translation(Path::new(subject), key, &code, options.is_detailed);
        }
      }

      for problem in TranslationTableCompiler::inspect(subject, parsed, language)? {
        xrf_output::error!(options.output, "{}: {}", subject, problem.message);

        result.problems.push(problem);
      }

      result.record_language_summary(TranslationVerifyLanguageSummary {
        file: subject.to_owned(),
        language: code,
        checked: parsed.len() as u32,
        missing,
      });
    }

    result.checked_translations_count = parsed.len() as u32;

    Ok(result)
  }

  /// What only the sources together make a problem: one id two of them define, which the game resolves by load order
  /// and xray-16 loads in parallel, and two building to one table, which the game opens by name alone.
  fn find_cross_source_problems(
    sources: &[(String, Vec<String>)],
    language: TranslationLanguage,
  ) -> XrfResult<Vec<TranslationFinding>> {
    let mut problems: Vec<TranslationFinding> = Vec::new();
    let mut defined: Vec<(&str, &str)> = sources
      .iter()
      .flat_map(|(subject, ids)| ids.iter().map(move |id| (id.as_str(), subject.as_str())))
      .collect();

    defined.sort();

    for pair in defined.windows(2) {
      if pair[0].0 == pair[1].0 {
        problems.push(TranslationFinding::new(
          "translations.duplicate-across-files",
          Some(pair[1].1.to_owned()),
          format!(
            "'{}' is also defined in '{}', and which one the game shows depends on load order",
            pair[0].0, pair[0].1
          ),
        ));
      }
    }

    let names: Vec<String> = sources.iter().map(|(subject, _)| subject.clone()).collect();

    if let Some((existing, source, target)) = TranslationBuildTargets::find_collision(&names, language)? {
      problems.push(TranslationFinding::new(
        "translations.collision",
        Some(source),
        format!(
          "Builds to '{}' as '{existing}' does, and the game loads tables by name alone",
          format_path(target.as_path())
        ),
      ));
    }

    Ok(problems)
  }
}
