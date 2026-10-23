use xrf_error::XrfResult;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayRoots, XraySearch};

use crate::json::read::parse_json;
use crate::language::TranslationLanguage;
use crate::project::descriptor::{
  TranslationFile, TranslationFinding, TranslationProjectDescriptor, TranslationProjectMode, TranslationSource,
};
use crate::source_file_name::is_json_source_name;
use crate::types::TranslationJson;

/// Read an XRF translations source tree.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted or the prefix is not a logical path. Individual
/// files are reported, not fatal.
pub fn read_source(roots: &XrayRoots, prefix: &str) -> XrfResult<TranslationProjectDescriptor> {
  read_source_in(&roots.to_search_plan()?.open()?, roots, prefix)
}

/// Read a source tree over roots somebody else mounted.
///
/// # Errors
///
/// Returns an error when the prefix is not a logical path.
pub fn read_source_in(search: &XraySearch, roots: &XrayRoots, prefix: &str) -> XrfResult<TranslationProjectDescriptor> {
  // An empty prefix means the root itself, for a caller that mounted the translations directory directly.
  let sources: XraySearch = search.clone().with_optional_prefix(Some(prefix))?;

  let mut descriptor: TranslationProjectDescriptor = TranslationProjectDescriptor {
    mode: TranslationProjectMode::Source,
    roots: roots.clone(),
    prefix: prefix.to_owned(),
    ..Default::default()
  };

  // Ordered by engine identity rather than mount order, so an index is comparable across runs and machines.
  let assets: Vec<XrayAsset> = sources.list_files();

  for asset in &assets {
    if is_json_source_name(asset.get_logical_path().file_name()) {
      merge_json(&sources, asset, prefix, &mut descriptor);
    }
  }

  record_cross_file_duplicates(&mut descriptor);

  for language in &descriptor.languages {
    // Source text is authored, not shipped, so the language decides the code page the build will have
    // to write it in.
    if let Ok(known) = TranslationLanguage::from_str_single(language) {
      descriptor
        .encodings
        .insert(language.clone(), known.get_language_encoding());
    }
  }

  descriptor.languages.sort();

  descriptor.finalize_editable();

  Ok(descriptor)
}

fn merge_json(search: &XraySearch, asset: &XrayAsset, prefix: &str, descriptor: &mut TranslationProjectDescriptor) {
  let logical_path: &str = asset.get_logical_path().as_str();

  // The per-file reader stays strict because the build and the verifier rely on it. Its refusal is
  // caught here instead, so one unreadable file costs its own strings and not the whole project.
  let translations: TranslationJson = match search
    .read_asset(asset)
    .and_then(|data| parse_json(logical_path, &data))
  {
    Ok(translations) => translations,
    Err(error) => {
      descriptor.findings.push(TranslationFinding::new(
        "translations.unreadable",
        Some(logical_path.to_owned()),
        format!("Could not read this file, so its strings are missing: {error}"),
      ));

      return;
    }
  };

  let source: TranslationSource = source_of(asset);
  let file: &mut TranslationFile = descriptor
    .files
    .entry(relative(prefix, asset.get_logical_path()))
    .or_default();

  for (id, languages) in translations {
    for language in languages.keys() {
      register_language(&mut descriptor.languages, language);

      // Every id in this file resolves to the same source, so record it the first time each language
      // is seen. Inserting per id instead would clone the pair of paths once per string in the file —
      // tens of thousands of times over an XRF source tree, to store the value already there.
      if !file.sources.contains_key(language) {
        file.sources.insert(language.clone(), source.clone());
      }
    }

    file.entries.insert(id, languages);
  }
}

/// One id served from two files is a conflict the engine resolves by load order, which is not
/// something a project should be relying on.
fn record_cross_file_duplicates(descriptor: &mut TranslationProjectDescriptor) {
  let mut seen: Vec<(String, String)> = Vec::new();

  for (key, file) in &descriptor.files {
    for id in file.entries.keys() {
      seen.push((id.clone(), key.clone()));
    }
  }

  seen.sort();

  for pair in seen.windows(2) {
    if pair[0].0 == pair[1].0 {
      descriptor.findings.push(TranslationFinding::new(
        "translations.duplicate-across-files",
        Some(pair[1].1.clone()),
        format!("'{}' is also defined in '{}'", pair[0].0, pair[0].1),
      ));
    }
  }
}

/// A logical path with the project's own prefix removed, which is how a file is keyed and shown.
fn relative(prefix: &str, logical_path: &XrayLogicalPath) -> String {
  if prefix.is_empty() {
    return logical_path.as_str().to_owned();
  }

  logical_path
    .strip_prefix(prefix)
    .ok()
    .flatten()
    .unwrap_or_else(|| logical_path.as_str())
    .to_owned()
}

fn source_of(asset: &XrayAsset) -> TranslationSource {
  TranslationSource::new(
    asset.get_logical_path().as_str(),
    asset.to_physical_path().as_deref().map(to_portable_path_string),
  )
}

fn register_language(languages: &mut Vec<String>, language: &str) {
  if !languages.iter().any(|known| known == language) {
    languages.push(language.to_owned());
  }
}
