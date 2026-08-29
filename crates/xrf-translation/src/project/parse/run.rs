use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use indexmap::IndexMap;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayLookupScope, XrayScopedVfs, XrayVfs};

use crate::json;
use crate::json::read::parse_json;
use crate::project::parse::merge::{MergeOutcome, merge_entries};
use crate::project::parse::options::ProjectParseOptions;
use crate::project::parse::result::ProjectParseResult;
use crate::project::parse::scope::{ResolvedParseScope, resolve};
use crate::staged_write::{write_file_staged, write_new_file};
use crate::types::TranslationJson;
use crate::xml;
use crate::xml::encoding::TranslationIdentity;
use crate::xml::read::parse_required_string_table;

/// One file's entries while duplicates are being resolved, in the order the table declared them.
type StringTableEntries = IndexMap<String, String>;

/// Import one language's raw string tables into JSON sources.
///
/// Reads through the VFS, so an installation imports exactly like a loose tree — on Anomaly and CoC
/// the tables live inside `db\configs`, and a reader reaching for the filesystem reports them absent
/// instead of reading them. Writes to the host, because JSON sources are files a translator edits and
/// there is nowhere to put one inside a volume.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted, when the scope still holds languages other than
/// the one declared, and when a target cannot be written. An individual unreadable table is a finding,
/// not a failure: one malformed file in a 136-file mod costs that file and not the import.
pub fn parse_translations(options: &ProjectParseOptions) -> XrfResult<ProjectParseResult> {
  let vfs: XrayVfs = options.roots.open()?;

  parse_translations_in(&vfs, options)
}

/// Import over roots somebody else mounted.
///
/// # Errors
///
/// The same as [`parse_translations`], minus mounting.
pub fn parse_translations_in(vfs: &XrayVfs, options: &ProjectParseOptions) -> XrfResult<ProjectParseResult> {
  let started_at: Instant = Instant::now();
  let language: String = options.language.to_string();

  let mut result: ProjectParseResult = ProjectParseResult::new(&language, options.is_dry_run);

  let scope: ResolvedParseScope = resolve(vfs, options.prefix.as_deref(), options.language)?;

  xrf_output::info!(
    options.output,
    "Parsing translations at {} as '{language}'{}",
    scope.describe(),
    if options.is_dry_run { " (dry run)" } else { "" }
  );

  let lookup: XrayLookupScope = XrayLookupScope::all().with_optional_prefix(Some(scope.prefix()))?;
  let scoped: XrayScopedVfs = vfs.scoped(&lookup);

  let mut assets: Vec<XrayAsset> = scoped
    .list_entries()
    .into_iter()
    .filter(|asset| asset.get_logical_path().has_extension(xml::FILE_EXTENSION_DOT))
    .filter(|asset| is_selected(asset, options.file.as_deref()))
    .collect();

  // Sorted because mount order is not name order, and a run is only comparable across machines if it
  // depends on neither.
  assets.sort_by(|left, right| left.get_logical_path().as_str().cmp(right.get_logical_path().as_str()));

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

  for asset in &assets {
    import_asset(&scoped, asset, &scope, &language, options, &mut result)?;
  }

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
  scoped: &XrayScopedVfs,
  asset: &XrayAsset,
  scope: &ResolvedParseScope,
  language: &str,
  options: &ProjectParseOptions,
  result: &mut ProjectParseResult,
) -> XrfResult {
  let logical_path: &str = asset.get_logical_path().as_str();

  // The declared language is supplied as the directory name, because that is the slot a string table
  // carries its language in. It only matters as an encoding fallback: a file that declares its code
  // page is read with that one either way.
  let identity: TranslationIdentity = TranslationIdentity {
    file_name: asset.get_logical_path().file_name(),
    directory_name: Some(language),
  };

  let entries: Vec<(String, String)> = match scoped
    .read_asset_bytes(asset)
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

  let entries: Vec<(String, String)> = deduplicate(entries, logical_path, result);
  let target: PathBuf = target_path(&options.output_dir, scope, logical_path)?;
  let existing: TranslationJson = read_existing(&target, result)?;

  let (merged, outcome): (TranslationJson, MergeOutcome) =
    merge_entries(existing, &entries, language, options.is_overwrite);

  outcome.record(&mut result.census);

  write_merged(&target, &merged, &outcome, options, result)
}

/// Keep the last of a repeated id, which is the one `CStringTable::Load` leaves in the table.
///
/// Reported rather than silently resolved: a file that defines an id twice is relying on load order,
/// and the strings that lose are invisible in game with nothing to say why.
fn deduplicate(
  entries: Vec<(String, String)>,
  logical_path: &str,
  result: &mut ProjectParseResult,
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

/// Where one string table's JSON lands, mirroring its path below the resolved scope.
///
/// Mirrored rather than flattened so two tables with the same name in different directories cannot
/// merge into one file. For the flat trees that gamedata and XRF sources actually are, this is the
/// same thing as flattening.
///
/// # Errors
///
/// Returns an invalid error when the logical path has no file name.
fn target_path(output_dir: &Path, scope: &ResolvedParseScope, logical_path: &str) -> XrfResult<PathBuf> {
  let relative: &str = logical_path
    .strip_prefix(scope.prefix())
    .map_or(logical_path, |rest| rest.trim_start_matches('\\'));

  let mut target: PathBuf = output_dir.to_path_buf();

  // Component by component, because a logical path is `\`-separated whatever the host uses.
  for component in relative.split('\\').filter(|component| !component.is_empty()) {
    target.push(component);
  }

  let stem: String = target
    .file_stem()
    .and_then(|stem| stem.to_str())
    .ok_or_else(|| XrfError::new_invalid_error(format!("Translation source '{logical_path}' has no usable file name")))?
    .to_owned();

  target.set_file_name(format!("{stem}.{}", json::FILE_EXTENSION));

  Ok(target)
}

/// What the target already holds, or an empty document when it holds nothing yet.
///
/// A target that exists but cannot be parsed stops this file rather than being overwritten: replacing
/// a JSON source somebody is editing because it briefly failed to parse would destroy the very work
/// the merge exists to protect.
fn read_existing(target: &Path, result: &mut ProjectParseResult) -> XrfResult<TranslationJson> {
  if !target.exists() {
    return Ok(TranslationJson::default());
  }

  match fs::read(target).map_err(XrfError::from).and_then(|data| {
    let subject: String = target.display().to_string();

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
  merged: &TranslationJson,
  outcome: &MergeOutcome,
  options: &ProjectParseOptions,
  result: &mut ProjectParseResult,
) -> XrfResult {
  let is_existing: bool = target.exists();

  if !outcome.is_changed {
    result.census.files_unchanged += 1;

    xrf_output::verbose!(options.output, "Unchanged {}", format_path(target));

    return Ok(());
  }

  if is_existing {
    result.census.files_updated += 1;
  } else {
    result.census.files_created += 1;
  }

  xrf_output::info!(
    options.output,
    "{} {}",
    if options.is_dry_run {
      "Would write"
    } else if is_existing {
      "Updating"
    } else {
      "Creating"
    },
    format_path(target)
  );

  if options.is_dry_run {
    return Ok(());
  }

  let mut serialized: Vec<u8> = serde_json::to_vec_pretty(merged).map_err(|error| {
    XrfError::new_serialization_error(format!(
      "Failed to serialize parsed translation JSON '{}': {error}",
      format_path(target)
    ))
  })?;

  // A new file gets one, an existing file keeps whatever convention it already had, which is what
  // `apply_edits` does — so the editor and the importer do not fight over it on alternate saves.
  if !is_existing || fs::read(target).is_ok_and(|original| original.ends_with(b"\n")) {
    serialized.push(b'\n');
  }

  if is_existing {
    write_file_staged(target, &serialized)
  } else {
    write_new_file(target, &serialized)
  }
}
