use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use crate::language::TranslationLanguage;
use crate::project::constants::{MAP_DESC_DIRECTORY, OPENXRAY_XML};
use crate::project::descriptor::{
  TranslationFile, TranslationFinding, TranslationProjectDescriptor, TranslationProjectMode,
};
use crate::types::{TranslationEntry, TranslationVariant};
use crate::xml;
use crate::xml::encoding::read_decoded;
use crate::xml::read::read_string_table;
use xrf_error::XrfResult;
use xrf_utils::to_portable_path_string;

/// Read a `text/` directory whose subdirectories are languages.
///
/// Language discovery mirrors `CStringTable::FillLanguageToken` rather than any list of our own: the
/// engine has no whitelist, so neither does this. A folder is a language unless it is named
/// `map_desc`, is empty, or holds nothing but `openxray.xml`.
///
/// # Errors
///
/// Returns an IO error when the root itself cannot be listed. Individual files never fail the read;
/// they are reported as findings instead.
pub fn read_gamedata<P: AsRef<Path>>(root: P) -> XrfResult<TranslationProjectDescriptor> {
  let root: &Path = root.as_ref();
  let mut descriptor: TranslationProjectDescriptor = TranslationProjectDescriptor {
    mode: TranslationProjectMode::Gamedata,
    root: root.to_string_lossy().into_owned(),
    ..Default::default()
  };

  for language in discover_languages(root, &mut descriptor.findings)? {
    let directory: PathBuf = root.join(&language);

    // Non-recursive, matching the engine's own `text\<language>\*.xml` mask.
    let mut names: Vec<String> = fs::read_dir(&directory)?
      .filter_map(Result::ok)
      .filter(|entry| entry.path().is_file())
      .filter(|entry| {
        Path::new(&entry.file_name())
          .extension()
          .is_some_and(|extension| extension.eq_ignore_ascii_case(xml::FILE_EXTENSION))
      })
      .map(|entry| entry.file_name().to_string_lossy().into_owned())
      .collect();

    names.sort();

    for name in names.iter() {
      merge_file(&directory.join(name), name, &language, &mut descriptor);
    }

    // Read off the first file rather than assumed from the code: these directories carry languages
    // the enum has no mapping for, and their own declaration is the only statement that exists.
    if let Some(encoding) = names
      .first()
      .and_then(|name| read_decoded(&directory.join(name)).ok())
      .map(|decoded| decoded.encoding.name().to_lowercase())
    {
      descriptor.encodings.insert(language.clone(), encoding);
    }

    descriptor.languages.push(language);
  }

  descriptor.files.sort_keys();

  Ok(descriptor)
}

fn merge_file(path: &Path, name: &str, language: &str, descriptor: &mut TranslationProjectDescriptor) {
  let subject: String = to_portable_path_string(path);

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

  let file: &mut TranslationFile = descriptor.files.entry(name.to_owned()).or_default();

  file.sources.insert(language.to_owned(), subject.clone());

  for (id, text) in entries {
    let entry: &mut TranslationEntry = file.entries.entry(id.clone()).or_default();

    // Last occurrence wins, because that is the one `CStringTable::Load` leaves in the table.
    if entry
      .insert(language.to_owned(), Some(TranslationVariant::String(text)))
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

fn discover_languages(root: &Path, findings: &mut Vec<TranslationFinding>) -> XrfResult<Vec<String>> {
  let mut languages: Vec<String> = Vec::new();

  for entry in fs::read_dir(root)? {
    let entry: DirEntry = entry?;

    if !entry.path().is_dir() {
      continue;
    }

    let name: String = entry.file_name().to_string_lossy().into_owned();

    if name == MAP_DESC_DIRECTORY {
      continue;
    }

    let files: Vec<PathBuf> = fs::read_dir(entry.path())?
      .filter_map(Result::ok)
      .map(|file| file.path())
      .filter(|path| path.is_file())
      .collect();

    if files.is_empty() {
      continue;
    }

    if files.len() == 1
      && files[0]
        .file_name()
        .is_some_and(|file_name| file_name.eq_ignore_ascii_case(OPENXRAY_XML))
    {
      continue;
    }

    if TranslationLanguage::from_str_single(&name).is_err() {
      findings.push(TranslationFinding::new(
        "translations.unknown-language",
        Some(to_portable_path_string(entry.path())),
        format!("'{name}' is a language the game loads but XRF does not build"),
      ));
    }

    languages.push(name);
  }

  languages.sort();

  Ok(languages)
}
