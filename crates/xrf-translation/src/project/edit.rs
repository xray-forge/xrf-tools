use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::XRayEncoding;
use xrf_vfs::XrayAsset;
use xrf_xml::encoding_from_label;

use crate::edit::TranslationEdit;
use crate::json;
use crate::language::find_unencodable_character;
use crate::project::descriptor::TranslationProjectDescriptor;
use crate::xml;

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
  match path.extension().and_then(|extension| extension.to_str()) {
    Some(json::FILE_EXTENSION) => json::write::apply_edits(path, language, edits),
    Some(xml::FILE_EXTENSION) => xml::write::apply_edits(path, edits),
    _ => Err(XrfError::new_invalid_error(format!(
      "Translation '{}' is not a file this can write",
      path.display()
    ))),
  }
}

/// Apply edits to one language's copy of a file, addressed by the asset the VFS resolved.
///
/// Takes the asset rather than the path the descriptor recorded, because that one is *portable* — a
/// form built for showing, and lossy by construction. `to_string_lossy` turns a name that is not valid
/// Unicode into replacement characters, and rewriting `\` as `/` changes what a name means on a host
/// where `\` is an ordinary filename character. Either sends a write to a path that is not the file,
/// with nothing to signal it. The asset still carries the real one.
///
/// Re-resolving also re-checks: a file shadowed since the project opened is written where the engine
/// would now read it, rather than to a copy that no longer wins.
///
/// The guard is the absence of a physical path, not a test of what kind of mount won. The refusal
/// itself belongs to `xrf-vfs`, so every editing domain answers this with one message and one kind.
///
/// # Errors
///
/// Returns an asset error naming the file when its winner is archived, and otherwise whatever
/// [`apply_edits`] answers with.
pub fn apply_edits_to_asset(asset: &XrayAsset, language: &str, edits: &[TranslationEdit]) -> XrfResult {
  apply_edits(&asset.to_writable_path()?, language, edits)
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
