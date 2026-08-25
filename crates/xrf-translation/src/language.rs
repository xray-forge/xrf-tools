use std::str::FromStr;

use derive_more::Display;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, new_windows1250_encoder, new_windows1251_encoder, new_windows1252_encoder};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Display)]
pub enum TranslationLanguage {
  #[display("all")]
  All,
  #[display("eng")]
  English,
  #[display("rus")]
  Russian,
  #[display("ukr")]
  Ukrainian,
  #[display("pol")]
  Polish,
  #[display("fra")]
  French,
  #[display("ger")]
  German,
  #[display("ita")]
  Italian,
  #[display("spa")]
  Spanish,
}

impl FromStr for TranslationLanguage {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "all" => Ok(Self::All),
      "eng" => Ok(Self::English),
      "rus" => Ok(Self::Russian),
      "ukr" => Ok(Self::Ukrainian),
      "pol" => Ok(Self::Polish),
      "fra" => Ok(Self::French),
      "ger" => Ok(Self::German),
      "ita" => Ok(Self::Italian),
      "spa" => Ok(Self::Spanish),
      _ => Err(format!("Unknown language: {}", s)),
    }
  }
}

impl TranslationLanguage {
  pub fn get_language_encoding(&self) -> String {
    match self {
      Self::Russian | Self::Ukrainian => String::from("windows-1251"),
      Self::German | Self::Polish => String::from("windows-1250"),
      _ => String::from("windows-1252"),
    }
  }

  pub fn new_language_encoder(&self) -> XRayEncoding {
    match self {
      Self::Russian | Self::Ukrainian => new_windows1251_encoder(),
      Self::German | Self::Polish => new_windows1250_encoder(),
      _ => new_windows1252_encoder(),
    }
  }

  pub fn get_all() -> Vec<Self> {
    vec![
      Self::English,
      Self::French,
      Self::German,
      Self::Italian,
      Self::Polish,
      Self::Russian,
      Self::Spanish,
      Self::Ukrainian,
    ]
  }

  pub fn get_all_strings() -> Vec<String> {
    Self::get_all().iter().map(|it| it.to_string()).collect()
  }

  pub fn from_str_single(language: &str) -> XrfResult<Self> {
    match Self::from_str(language).map_err(|it| XrfError::new_parsing_error(it.to_string()))? {
      Self::All => Err(XrfError::new_unknown_language_error(String::from(
        "Unexpected language 'all' provided'",
      ))),
      language => Ok(language),
    }
  }

  /// Read the language off the `rus` of a `text/rus/st_dialogs.xml`, which is how gamedata carries it.
  pub(crate) fn from_directory_name(directory_name: &str) -> Option<Self> {
    Self::from_str_single(directory_name).ok()
  }
}

/// Find the first character an encoding cannot represent.
///
/// Lives beside the languages because it answers a question about a code page, and every caller -
/// the builder, both writers, and edit validation - is asking it about a language's own encoding.
pub(crate) fn find_unencodable_character(value: &str, encoding: XRayEncoding) -> Option<char> {
  value
    .chars()
    .find(|character| encoding.encode(&String::from(*character)).2)
}

#[cfg(test)]
mod tests;
