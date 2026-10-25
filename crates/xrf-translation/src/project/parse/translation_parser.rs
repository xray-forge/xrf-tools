use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use indexmap::IndexMap;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::{HostDestination, HostFileName, HostRelativePath, format_path};
use xrf_vfs::{XrayAsset, XraySearch};

use crate::json::read::parse_json;
use crate::json::write::{CanonicalRender, render_canonical, write_canonical};
use crate::project::parse::merge::{MergeOutcome, merge_entries};
use crate::project::parse::scope::{ResolvedParseScope, resolve};
use crate::project::parse::translation_parse_options::TranslationParseOptions;
use crate::project::parse::translation_parse_result::TranslationParseResult;
use crate::types::TranslationJson;
use crate::xml::read::parse_required_string_table;
use crate::xml::translation_identity::TranslationIdentity;

/// One file's entries while duplicates are being resolved, in the order the table declared them.
type StringTableEntries = IndexMap<String, String>;

/// Imports raw X-Ray XML string tables into multi-language JSON sources.
pub struct TranslationParser;

impl TranslationParser {
  /// Import one language's raw string tables into JSON sources.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, when the scope still holds languages other than
  /// the one declared, and when a target cannot be written. An individual unreadable table is a finding,
  /// not a failure: one malformed file in a 136-file mod costs that file and not the import.
  pub fn parse(options: &TranslationParseOptions) -> XrfResult<TranslationParseResult> {
    Self::parse_in(&options.roots.to_search_plan()?.open()?, options)
  }

  /// Import over roots somebody else mounted.
  ///
  /// # Errors
  ///
  /// The same as [`Self::parse`], minus mounting.
  pub fn parse_in(search: &XraySearch, options: &TranslationParseOptions) -> XrfResult<TranslationParseResult> {
    let started_at: Instant = Instant::now();
    let language: String = options.language.to_string();

    let mut result: TranslationParseResult = TranslationParseResult::new(&language, options.is_dry_run);

    let scope: ResolvedParseScope = resolve(search, options.prefix.as_deref(), options.language)?;

    xrf_output::info!(
      options.output,
      "Parsing translations at {} as '{language}'{}",
      scope.describe(),
      if options.is_dry_run { " (dry run)" } else { "" }
    );

    let tables: XraySearch = search.clone().with_optional_prefix(Some(scope.prefix()))?;

    // Ordered by engine identity rather than mount order, so a run is comparable across machines.
    let assets: Vec<XrayAsset> = tables
      .list_files()
      .into_iter()
      .filter(|asset| asset.get_logical_path().has_extension(XrayExtension::Xml))
      .filter(|asset| Self::is_selected(asset, options.file.as_deref()))
      .collect();

    // A run that found nothing is refused rather than reported as a success that wrote no files. A
    // mistyped prefix, a `--file` that names nothing, and a root holding no tables all land here, and
    // all of them look identical to a clean import if the answer is exit 0 with an empty census — which
    // is the exact failure this command was reported for in the first place.
    if assets.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "No string tables to import at '{}'{}. Check the path, the language, and --prefix.",
        scope.describe(),
        options
          .file
          .as_deref()
          .map_or(String::new(), |file| format!(" matching '{file}'")),
      )));
    }

    let destination: HostDestination = HostDestination::new(&options.output_dir, options.link_policy);

    let parsing: xrf_job::JobScope = options.job.enter(
      crate::project::job_phases::TRANSLATION_PHASE_PARSE,
      Some(assets.len() as u64),
    );

    for asset in &assets {
      // Between tables: each source is written whole through a staged replace, so stopping leaves the ones already
      // written complete and the rest untouched.
      if options.job.is_cancelled() {
        result.outcome = xrf_job::JobOutcome::Cancelled;

        break;
      }

      options
        .job
        .set_detail(Some(asset.get_logical_path().as_str().to_owned()));

      Self::import_asset(&tables, asset, &scope, &language, &destination, options, &mut result)?;

      parsing.advance();
    }

    options.job.set_detail(None);

    result.finalize(started_at.elapsed());

    log::info!(
      "Parsed {} translation file(s) as '{language}' in {}",
      result.census.files_read,
      xrf_utils::format_duration(result.duration)
    );

    Ok(result)
  }

  /// Read one string table and merge it into the JSON source it belongs to.
  fn import_asset(
    search: &XraySearch,
    asset: &XrayAsset,
    scope: &ResolvedParseScope,
    language: &str,
    destination: &HostDestination,
    options: &TranslationParseOptions,
    result: &mut TranslationParseResult,
  ) -> XrfResult {
    let logical_path: &str = asset.get_logical_path().as_str();

    // The declared language is supplied as the directory name, because that is the slot a string table
    // carries its language in. It only matters as an encoding fallback: a file that declares its code
    // page is read with that one either way.
    let identity: TranslationIdentity = TranslationIdentity {
      file_name: asset.get_logical_path().file_name(),
      directory_name: Some(language),
    };

    let entries: Vec<(String, String)> = match search
      .read_asset(asset)
      .and_then(|data| parse_required_string_table(identity, &data))
    {
      Ok(entries) => entries,
      Err(error) => {
        result.census.files_skipped += 1;
        result.record_finding(
          "translations.unreadable",
          logical_path,
          format!("Could not read this file, so its strings were not imported: {error}"),
        );

        return Ok(());
      }
    };

    if entries.is_empty() {
      result.census.files_skipped += 1;
      result.record_finding(
        "translations.empty",
        logical_path,
        "This string table holds no entries, so no source was written for it",
      );

      return Ok(());
    }

    result.census.files_read += 1;
    result.census.entries_read += entries.len() as u32;

    let entries: Vec<(String, String)> = Self::deduplicate(entries, logical_path, result);
    let relative: HostRelativePath = Self::target_path(scope, logical_path)?;
    let target: PathBuf = destination.resolve_file(&relative)?;
    let existing: TranslationJson = Self::read_existing(&target, result)?;

    let (merged, outcome): (TranslationJson, MergeOutcome) =
      merge_entries(existing, &entries, language, options.is_overwrite);

    outcome.record(&mut result.census);

    Self::write_merged(&target, &relative, destination, &merged, options, result)
  }

  /// Keep the last of a repeated id, which is the one `CStringTable::Load` leaves in the table.
  fn deduplicate(
    entries: Vec<(String, String)>,
    logical_path: &str,
    result: &mut TranslationParseResult,
  ) -> Vec<(String, String)> {
    let mut deduplicated: StringTableEntries = StringTableEntries::default();

    for (id, text) in entries {
      if deduplicated.insert(id.clone(), text).is_some() {
        result.record_finding(
          "translations.duplicate",
          logical_path,
          format!("'{id}' appears more than once; the game uses the last one and the others are ignored"),
        );
      }
    }

    deduplicated.into_iter().collect()
  }

  /// Whether a file was asked for, when the run was narrowed to one table.
  ///
  /// Matched on the name rather than the path, so `--file st_items.xml` means the same thing whichever
  /// directory depth it sits at, and case-insensitively, because the VFS lower-cases logical paths while
  /// the caller types whatever the host shows them.
  fn is_selected(asset: &XrayAsset, file: Option<&str>) -> bool {
    match file {
      Some(name) => asset.get_logical_path().file_name().eq_ignore_ascii_case(name),
      None => true,
    }
  }

  /// Where one string table's JSON lands below the output directory, mirroring its path below the resolved scope.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the logical path has no file name or would leave the output directory.
  fn target_path(scope: &ResolvedParseScope, logical_path: &str) -> XrfResult<HostRelativePath> {
    let relative: HostRelativePath = HostRelativePath::new(
      logical_path
        .strip_prefix(scope.prefix())
        .map_or(logical_path, |rest| rest.trim_start_matches('\\')),
      "Translation table",
    )?;

    let stem: &str = relative
      .as_path()
      .file_stem()
      .and_then(|stem| stem.to_str())
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!("Translation source '{logical_path}' has no usable file name"))
      })?;

    let file: HostFileName = HostFileName::new(stem, "Translation table")?.with_extension(XrayExtension::Json.as_str());

    Ok(relative.parent().unwrap_or_default().join(&file))
  }

  /// What the target already holds, or an empty document when it holds nothing yet.
  fn read_existing(target: &Path, result: &mut TranslationParseResult) -> XrfResult<TranslationJson> {
    if !target.exists() {
      return Ok(TranslationJson::default());
    }

    match fs::read(target).map_err(XrfError::from).and_then(|data| {
      let subject: String = format_path(target).to_string();

      parse_json(&subject, &data)
    }) {
      Ok(existing) => Ok(existing),
      Err(error) => {
        result.record_path_finding(
          "translations.unmergeable",
          target,
          format!("Could not read the existing source, so nothing was merged into it: {error}"),
        );

        Err(error)
      }
    }
  }

  /// Write the merged document, unless nothing changed or the run was told not to.
  fn write_merged(
    target: &Path,
    relative: &HostRelativePath,
    destination: &HostDestination,
    merged: &TranslationJson,
    options: &TranslationParseOptions,
    result: &mut TranslationParseResult,
  ) -> XrfResult {
    let render: CanonicalRender = render_canonical(target, merged, None)?;

    if !render.is_changed() {
      result.census.files_unchanged += 1;

      xrf_output::verbose!(options.output, "Unchanged {}", format_path(target));

      return Ok(());
    }

    let is_new: bool = render.is_new();

    if is_new {
      result.census.files_created += 1;
    } else {
      result.census.files_updated += 1;
    }

    xrf_output::info!(
      options.output,
      "{} {}",
      if options.is_dry_run {
        "Would write"
      } else if is_new {
        "Creating"
      } else {
        "Updating"
      },
      format_path(target)
    );

    if options.is_dry_run {
      return Ok(());
    }

    // The directories are made here rather than by the staged writer, so the run's link policy decides each of them.
    destination.create_root()?;
    destination.prepare_file(relative)?;

    write_canonical(target, &render)?;

    Ok(())
  }
}
