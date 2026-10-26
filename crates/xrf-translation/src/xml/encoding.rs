use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling, new_utf8_encoder};
use xrf_xml::XmlEncoding;

use crate::language::TranslationLanguage;
use crate::xml::decoded_translation::DecodedTranslation;
use crate::xml::translation_identity::TranslationIdentity;

/// Decide which encoding a string table file is written in.
///
/// Shared by reading and writing on purpose: resolving it differently in each direction would decode
/// a file one way and re-encode it another, corrupting every byte an edit never touched.
///
/// The declaration wins where there is one. Otherwise the language comes from the directory name,
/// which is where raw gamedata carries it and what an importer declares for a tree that carries none.
///
/// # Errors
///
/// Returns an encoding error when the declaration names a code page there is no encoder for.
pub(crate) fn resolve_encoding(identity: TranslationIdentity, data: &[u8]) -> XrfResult<XRayEncoding> {
  let language: Option<TranslationLanguage> = identity.get_language();
  let declared: Option<XRayEncoding> = XmlEncoding::find_declared(data)?;

  if let (Some(declared), Some(language)) = (declared, language)
    && declared != language.new_language_encoder()
  {
    log::warn!(
      "Translation XML '{}' declares {}, but '{}' expects {}",
      identity.file_name,
      declared.name(),
      language,
      language.get_language_encoding(),
    );
  }

  Ok(declared.unwrap_or_else(|| language.unwrap_or(TranslationLanguage::English).new_language_encoder()))
}

/// Decode a string table already in hand, such as an entry read out of an archive.
///
/// # Errors
///
/// Returns an encoding error when the bytes are UTF-16 or do not decode as the encoding they claim.
pub(crate) fn decode(identity: TranslationIdentity, data: &[u8]) -> XrfResult<DecodedTranslation> {
  let (mark, body) = split_byte_order_mark(identity.file_name, data)?;

  // A byte order mark decides the encoding and outranks the declaration, which shipped files
  // contradict: gamedata-coc's st_items_weapons.xml is UTF-8 marked and declares windows-1251. It is
  // also kept out of the decoded text and put back verbatim, because the usual decode strips it and
  // re-encoding would then drop it from a file that had one.
  let encoding: XRayEncoding = if mark.is_empty() {
    resolve_encoding(identity, body)?
  } else {
    new_utf8_encoder()
  };

  Ok(DecodedTranslation {
    byte_order_mark: mark.to_vec(),
    encoding,
    text: decode_bytes_to_string_without_bom_handling(body, encoding)?,
  })
}

/// Read a string table off disk and decode it.
///
/// For a caller that holds one file and no mounted roots; everything reading a tree goes through the
/// VFS and calls [`decode`] with the bytes it already has.
///
/// # Errors
///
/// Returns an IO error when the file cannot be read, and an encoding error when its bytes do not
/// decode as the encoding it claims.
pub(crate) fn read_decoded(path: &Path) -> XrfResult<DecodedTranslation> {
  decode(TranslationIdentity::from_path(path), &fs::read(path)?)
}

/// Split a leading byte order mark off the content, so it survives a rewrite untouched.
///
/// # Errors
///
/// Returns an encoding error for UTF-16 content, which none of the string table encoders can hold
/// and which decoding as a code page would silently mangle.
fn split_byte_order_mark<'a>(subject: &str, data: &'a [u8]) -> XrfResult<(&'a [u8], &'a [u8])> {
  if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
    return Ok(data.split_at(3));
  }

  if data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]) {
    return Err(XrfError::new_encoding_error(format!(
      "Translation '{subject}' is UTF-16, which string tables cannot be written as"
    )));
  }

  Ok((&[], data))
}
