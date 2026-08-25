use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling, new_utf8_encoder};
use xrf_xml::declared_xml_encoding;

use crate::language::TranslationLanguage;
use crate::source_file_name::TranslationSourceFileName;

/// Where a string table says which language it holds.
///
/// Two names rather than a path, because that is all the rule needs and because the two path domains
/// spell a path differently: an engine identity is `\`-separated where a host path is not, so a
/// reader working in either domain can supply these without converting between them.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TranslationIdentity<'a> {
  /// `st_dialogs.eng.xml`, which is how a source carries its language.
  pub file_name: &'a str,
  /// The `rus` of `text\rus\st_dialogs.xml`, which is how gamedata carries it.
  pub directory_name: Option<&'a str>,
}

impl<'a> TranslationIdentity<'a> {
  /// Read both names off a host path.
  pub(crate) fn from_path(path: &'a Path) -> Self {
    Self {
      file_name: path.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
      directory_name: path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str()),
    }
  }

  /// The language these names imply, if any.
  fn get_language(&self) -> Option<TranslationLanguage> {
    TranslationSourceFileName::parse(self.file_name)
      .and_then(|source_name| source_name.get_xml_language())
      .or_else(|| self.directory_name.and_then(TranslationLanguage::from_directory_name))
  }
}

/// A string table's bytes, decoded, with what is needed to write them back.
pub(crate) struct DecodedTranslation {
  pub byte_order_mark: Vec<u8>,
  pub encoding: XRayEncoding,
  pub text: String,
}

/// Decide which encoding a string table file is written in.
///
/// Shared by reading and writing on purpose: resolving it differently in each direction would decode
/// a file one way and re-encode it another, corrupting every byte an edit never touched.
///
/// The declaration wins where there is one. Otherwise the language comes from the filename suffix,
/// then from the parent directory, which is where raw gamedata carries it.
///
/// # Errors
///
/// Returns an encoding error when the declaration names a code page there is no encoder for.
pub(crate) fn resolve_encoding(identity: TranslationIdentity, data: &[u8]) -> XrfResult<XRayEncoding> {
  let language: Option<TranslationLanguage> = identity.get_language();
  let declared: Option<XRayEncoding> = declared_xml_encoding(data)?;

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
