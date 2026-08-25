use std::path::{Path, PathBuf};

use crate::json::read::read_json;
use crate::language::TranslationLanguage;
use crate::project::constants::{LANGUAGE_NEUTRAL, MULTILANGUAGE};
use crate::project::descriptor::{
  TranslationFile, TranslationFinding, TranslationProjectDescriptor, TranslationProjectMode,
};
use crate::project::layout::relative;
use crate::source_file_name::{TranslationSourceFileKind, TranslationSourceFileName};
use crate::types::{TranslationEntry, TranslationJson, TranslationVariant};
use crate::xml::read::read_string_table;
use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_portable_path_string;

/// Read an XRF translations source tree.
///
/// Language lives in a JSON key or an `.eng.xml` style filename here, so the same file can carry
/// several languages and several files can carry one between them. Unlike the build and the verifier,
/// nothing in here refuses to open on content: problems come back as findings.
///
/// # Errors
///
/// Returns an IO error when the tree cannot be walked. Individual files are reported, not fatal.
pub fn read_source<P: AsRef<Path>>(root: P) -> XrfResult<TranslationProjectDescriptor> {
  let root: &Path = root.as_ref();
  let mut descriptor: TranslationProjectDescriptor = TranslationProjectDescriptor {
    mode: TranslationProjectMode::Source,
    root: root.to_string_lossy().into_owned(),
    ..Default::default()
  };

  for entry in WalkDir::new(root).sort_by_file_name() {
    let entry: DirEntry = entry.map_err(|error| {
      XrfError::new_read_error(format!(
        "Failed to walk translation directory '{}': {error}",
        root.display()
      ))
    })?;
    let path: &Path = entry.path();

    if !path.is_file() {
      continue;
    }

    if let Some(source_name) = path.file_name().and_then(TranslationSourceFileName::parse) {
      match source_name.get_kind() {
        TranslationSourceFileKind::Json => merge_json(root, path, &mut descriptor),
        TranslationSourceFileKind::Xml => merge_xml(root, path, &mut descriptor),
      }
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

  descriptor.languages.sort_by(|first, second| {
    // Neutral text belongs to no language, so it sorts last rather than under "a".
    (first == LANGUAGE_NEUTRAL, first).cmp(&(second == LANGUAGE_NEUTRAL, second))
  });

  Ok(descriptor)
}

fn merge_json(root: &Path, path: &Path, descriptor: &mut TranslationProjectDescriptor) {
  let subject: String = to_portable_path_string(path);

  // The per-file reader stays strict because the build and the verifier rely on it. Its refusal is
  // caught here instead, so one unreadable file costs its own strings and not the whole project.
  let translations: TranslationJson = match read_json(path) {
    Ok(translations) => translations,
    Err(error) => {
      descriptor.findings.push(TranslationFinding::new(
        "translations.unreadable",
        Some(subject),
        format!("Could not read this file, so its strings are missing: {error}"),
      ));

      return;
    }
  };

  let file: &mut TranslationFile = descriptor.files.entry(relative(root, path)).or_default();

  for (id, languages) in translations {
    for language in languages.keys() {
      register_language(&mut descriptor.languages, language);
      file.sources.insert(language.clone(), subject.clone());
    }

    file.entries.insert(id, languages);
  }
}

fn merge_xml(root: &Path, path: &Path, descriptor: &mut TranslationProjectDescriptor) {
  let subject: String = to_portable_path_string(path);

  // No language suffix means the build copies this file to every language, so presenting it as
  // English - which a filename fallback would do - would show a change to all of them as one.
  let language: String = path.file_name().and_then(TranslationSourceFileName::parse).map_or_else(
    || LANGUAGE_NEUTRAL.to_owned(),
    |source_name| {
      source_name
        .get_xml_language()
        .map_or_else(|| LANGUAGE_NEUTRAL.to_owned(), |language| language.to_string())
    },
  );

  let entries: Vec<(String, String)> = match read_string_table(path) {
    Ok(entries) => entries,
    Err(error) => {
      descriptor.findings.push(TranslationFinding::new(
        "translations.unreadable",
        Some(subject),
        format!("Could not read this file, so its strings are missing: {error}"),
      ));

      return;
    }
  };

  register_language(&mut descriptor.languages, &language);

  let file: &mut TranslationFile = descriptor.files.entry(xml_key(root, path, &language)).or_default();

  file.sources.insert(language.clone(), subject.clone());

  for (id, text) in entries {
    let entry: &mut TranslationEntry = file.entries.entry(id.clone()).or_default();

    if entry
      .insert(language.clone(), Some(TranslationVariant::String(text)))
      .is_some()
    {
      descriptor.findings.push(TranslationFinding::new(
        "translations.duplicate",
        Some(subject.clone()),
        format!("'{id}' appears more than once; the game uses the last one and the others are ignored"),
      ));
    }
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

/// Group the language-suffixed variants of one file under a single entry.
fn xml_key(root: &Path, path: &Path, language: &str) -> String {
  if language == LANGUAGE_NEUTRAL {
    return relative(root, path);
  }

  let merged: PathBuf = path.with_file_name(
    path
      .file_name()
      .and_then(|name| name.to_str())
      .map_or_else(String::new, |name| {
        name.replace(&format!(".{language}.xml"), &format!(".{MULTILANGUAGE}.xml"))
      }),
  );

  relative(root, &merged)
}

fn register_language(languages: &mut Vec<String>, language: &str) {
  if !languages.iter().any(|known| known == language) {
    languages.push(language.to_owned());
  }
}
