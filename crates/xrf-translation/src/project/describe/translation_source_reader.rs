use xrf_error::XrfResult;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayRoots, XraySearch};

use crate::json::translation_json_reader::TranslationJsonReader;
use crate::json::translation_source_name::TranslationSourceName;
use crate::language::translation_language::TranslationLanguage;
use crate::project::describe::translation_file::TranslationFile;
use crate::project::describe::translation_finding::TranslationFinding;
use crate::project::describe::translation_project_descriptor::TranslationProjectDescriptor;
use crate::project::describe::translation_project_mode::TranslationProjectMode;
use crate::project::describe::translation_source::TranslationSource;
use crate::text::translation_json::TranslationJson;

/// Reads an XRF translations source tree: multi-language JSON, one file per string table.
pub struct TranslationSourceReader;

impl TranslationSourceReader {
  /// Read a source tree over roots this mounts.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted or the prefix is not a logical path. Individual files are
  /// reported, not fatal.
  pub fn read(roots: &XrayRoots, prefix: &str) -> XrfResult<TranslationProjectDescriptor> {
    Self::read_in(&roots.to_search_plan()?.open()?, roots, prefix)
  }

  /// Read a source tree over roots somebody else mounted.
  ///
  /// # Errors
  ///
  /// Returns an error when the prefix is not a logical path.
  pub fn read_in(search: &XraySearch, roots: &XrayRoots, prefix: &str) -> XrfResult<TranslationProjectDescriptor> {
    // An empty prefix means the root itself, for a caller that mounted the translations directory directly.
    let sources: XraySearch = search.clone().with_optional_prefix(Some(prefix))?;

    let mut descriptor: TranslationProjectDescriptor = TranslationProjectDescriptor {
      mode: TranslationProjectMode::Source,
      roots: roots.clone(),
      prefix: prefix.to_owned(),
      ..Default::default()
    };

    // Ordered by engine identity rather than mount order, so an index is comparable across runs and machines.
    for asset in &sources.list_files() {
      if TranslationSourceName::is_name(asset.get_logical_path().file_name()) {
        Self::merge_json(&sources, asset, prefix, &mut descriptor);
      }
    }

    Self::record_cross_file_duplicates(&mut descriptor);

    for language in &descriptor.languages {
      // Source text is authored, not shipped, so the language decides the code page the build will have to write it
      // in.
      if let Ok(known) = TranslationLanguage::from_str_single(language) {
        descriptor
          .encodings
          .insert(language.clone(), known.get_encoding_label().to_owned());
      }
    }

    descriptor.languages.sort();

    descriptor.finalize_editable();

    Ok(descriptor)
  }

  fn merge_json(search: &XraySearch, asset: &XrayAsset, prefix: &str, descriptor: &mut TranslationProjectDescriptor) {
    let logical_path: &str = asset.get_logical_path().as_str();

    // The per-file reader stays strict because the build and the verifier rely on it. Its refusal is caught here
    // instead, so one unreadable file costs its own strings and not the whole project.
    let translations: TranslationJson = match search
      .read_asset(asset)
      .and_then(|data| TranslationJsonReader::parse(logical_path, &data))
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

    let source: TranslationSource = TranslationSource::new(
      logical_path,
      asset.to_physical_path().as_deref().map(to_portable_path_string),
    );
    let file: &mut TranslationFile = descriptor
      .files
      .entry(Self::relative(prefix, asset.get_logical_path()))
      .or_default();

    for (id, languages) in translations {
      for language in languages.keys() {
        if !descriptor.languages.iter().any(|known| known == language) {
          descriptor.languages.push(language.clone());
        }

        // Every id in this file resolves to the same source, so record it the first time each language is seen.
        // Inserting per id instead would clone the pair of paths once per string in the file — tens of thousands of
        // times over an XRF source tree, to store the value already there.
        if !file.sources.contains_key(language) {
          file.sources.insert(language.clone(), source.clone());
        }
      }

      file.entries.insert(id, languages);
    }
  }

  /// One id served from two files is a conflict the engine resolves by load order, which xray-16 makes no promise
  /// about at all: it loads string tables in parallel.
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
}
