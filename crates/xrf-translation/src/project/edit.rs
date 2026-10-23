use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_extension::{XrayExtension, XrayExtensionOf};
use xrf_utils::{XRayEncoding, format_path};
use xrf_vfs::{XrayAsset, XrayLogicalPath, XraySearch, XrayTransaction};
use xrf_xml::encoding_from_label;

use crate::edit::TranslationEdit;
use crate::language::find_unencodable_character;
use crate::project::descriptor::{TranslationProjectDescriptor, TranslationSource};
use crate::{json, xml};

/// Apply edits to whichever kind of translation file holds them.
///
/// The language matters for JSON, where one file carries all of them, and is already decided by the
/// path for XML, where the filename or the parent directory says which language it is.
///
/// # Errors
///
/// Returns a parsing error for an unreadable file, an encoding error for a value the target cannot
/// represent, an IO error when the file cannot be replaced, and an invalid error for a file this does
/// not know how to write.
pub fn apply_edits(path: &Path, language: &str, edits: &[TranslationEdit]) -> XrfResult {
  match XrayExtensionOf::of_path(path).known() {
    Some(XrayExtension::Json) => json::write::apply_edits(path, language, edits),
    Some(XrayExtension::Xml) => xml::write::apply_edits(path, edits),
    _ => Err(XrfError::new_invalid_error(format!(
      "Translation '{}' is not a file this can write",
      format_path(path)
    ))),
  }
}

/// One transaction saving a file's pending edits, by language, over what `search` reads of each language's copy now.
///
/// Each copy is rendered from its current bytes ([`render_edits`]); a JSON source holds every language, so its
/// languages apply one over the other, in language order so the file comes out the same however the edits arrived. A
/// copy is saved where it is: an archived one is refused, as the project's editability promised, rather than
/// overridden by a loose copy the commit would create.
///
/// # Errors
///
/// Returns a not-found error when a language has no copy, or its copy is no longer in `search`; an asset error naming
/// an archived copy; and whatever reading or rendering a copy answers.
pub fn build_save_transaction(
  search: &XraySearch,
  file: &str,
  sources: &HashMap<String, TranslationSource>,
  edits: &HashMap<String, Vec<TranslationEdit>>,
) -> XrfResult<XrayTransaction> {
  let mut languages: Vec<(&String, &Vec<TranslationEdit>)> =
    edits.iter().filter(|(_, edits)| !edits.is_empty()).collect();
  let mut rewritten: BTreeMap<XrayLogicalPath, Vec<u8>> = BTreeMap::new();

  languages.sort_by_key(|(language, _)| *language);

  for (language, language_edits) in languages {
    let source: &TranslationSource = sources.get(language).ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "Translations file '{file}' has nothing on disk for '{language}'"
      ))
    })?;
    let asset: XrayAsset = search.find(&source.logical_path)?.ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "Translations file '{}' is no longer in the mounted roots",
        source.logical_path
      ))
    })?;

    asset.to_writable_path()?;

    log::info!(
      "Saving {} edits to {} ({language})",
      language_edits.len(),
      source.logical_path
    );

    let existing: Vec<u8> = match rewritten.remove(asset.get_logical_path()) {
      Some(bytes) => bytes,
      None => search.read_asset(&asset)?,
    };
    let bytes: Vec<u8> = render_edits(asset.get_logical_path(), &existing, language, language_edits)?;

    rewritten.insert(asset.get_logical_path().clone(), bytes);
  }

  rewritten
    .into_iter()
    .try_fold(XrayTransaction::new(search), |transaction, (path, bytes)| {
      transaction.with_write(path.as_str(), bytes)
    })
}

/// The bytes one language's copy of a file holds once `edits` are applied to `existing`, what it holds now: the
/// rewrite [`apply_edits`] makes, for a caller that commits it through the VFS rather than writing it in place.
///
/// Addressed by logical path rather than the path the descriptor recorded, because that one is *portable* — a form
/// built for showing, and lossy by construction — and because the commit, not this, decides which file is written.
/// The logical path says which kind of file it is and, for XML, which language.
///
/// # Errors
///
/// Returns a parsing error for an unreadable file, an encoding error for a value the file cannot represent, and an
/// invalid error for a file this does not know how to write.
pub fn render_edits(
  logical_path: &XrayLogicalPath,
  existing: &[u8],
  language: &str,
  edits: &[TranslationEdit],
) -> XrfResult<Vec<u8>> {
  let path: PathBuf = logical_path.to_host_relative_path();

  match XrayExtensionOf::of_path(&path).known() {
    Some(XrayExtension::Json) => json::write::render_edits(logical_path.as_str(), existing, language, edits),
    Some(XrayExtension::Xml) => xml::write::render_edits(&path, existing, edits),
    _ => Err(XrfError::new_invalid_error(format!(
      "Translation '{logical_path}' is not a file this can write"
    ))),
  }
}

/// Report the first character a language cannot hold, or nothing when the value is writable.
///
/// Takes the descriptor because the answer depends on what each language's own files declared when
/// the project was read, which is the only statement that exists for a language XRF does not build.
///
/// # Errors
///
/// Returns an encoding error when a language declares a code page there is no encoder for.
pub fn find_unwritable_character(
  descriptor: &TranslationProjectDescriptor,
  language: &str,
  text: &str,
) -> XrfResult<Option<String>> {
  let candidates: Vec<(String, XRayEncoding)> = match descriptor.encodings.get(language) {
    Some(label) => vec![(language.to_owned(), encoding_from_label(label)?)],
    None => Vec::new(),
  };

  for (name, encoding) in candidates {
    if let Some(character) = find_unencodable_character(text, encoding) {
      return Ok(Some(format!(
        "'{character}' (U+{:04X}) cannot be written in {name}",
        character as u32
      )));
    }
  }

  Ok(None)
}
