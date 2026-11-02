use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, new_windows1250_encoder, new_windows1251_encoder, new_windows1252_encoder};

/// A language a translation is written in, spelled on the wire as its code; `all` stands for every one at once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Display, Serialize, Deserialize)]
pub enum TranslationLanguage {
  #[display("all")]
  #[serde(rename = "all")]
  All,
  #[display("eng")]
  #[serde(rename = "eng")]
  English,
  #[display("rus")]
  #[serde(rename = "rus")]
  Russian,
  #[display("ukr")]
  #[serde(rename = "ukr")]
  Ukrainian,
  #[display("pol")]
  #[serde(rename = "pol")]
  Polish,
  #[display("fra")]
  #[serde(rename = "fra")]
  French,
  #[display("ger")]
  #[serde(rename = "ger")]
  German,
  #[display("ita")]
  #[serde(rename = "ita")]
  Italian,
  #[display("spa")]
  #[serde(rename = "spa")]
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
  /// Every single language, in code order: what [`Self::All`] stands for.
  pub const LANGUAGES: [Self; 8] = [
    Self::English,
    Self::French,
    Self::German,
    Self::Italian,
    Self::Polish,
    Self::Russian,
    Self::Spanish,
    Self::Ukrainian,
  ];

  /// The single languages this stands for: every one for [`Self::All`], itself otherwise.
  pub fn expand(self) -> Vec<Self> {
    match self {
      Self::All => Self::LANGUAGES.to_vec(),
      language => vec![language],
    }
  }

  /// The code page the game draws this language's text in, which is the one its string tables are written in.
  ///
  /// The engine never decodes a table: `CStringTable` keeps its bytes, and the font maps each byte to a glyph. Which
  /// font is chosen by a prefix (`CStringTable::SetLanguage`, `CGameFont::Initialize`): `_west`, Windows-1252, for
  /// French, German, Italian and Spanish; `_cent`, Windows-1250, for Polish; and the unprefixed fonts, Windows-1251,
  /// for the rest, which is how xrf-engine and the stock `localization.ltx` files draw English, Russian and Ukrainian.
  /// A table's own `encoding` declaration plays no part, and shipped tables contradict it: Clear Sky's Polish, Czech,
  /// Hungarian and German tables all declare Windows-1251. [`Self::All`] stands for no one language and answers the
  /// unprefixed fonts'.
  pub fn get_encoding(self) -> XRayEncoding {
    match self {
      Self::French | Self::German | Self::Italian | Self::Spanish => new_windows1252_encoder(),
      Self::Polish => new_windows1250_encoder(),
      Self::All | Self::English | Self::Russian | Self::Ukrainian => new_windows1251_encoder(),
    }
  }

  /// The label of [`Self::get_encoding`]'s code page, as a string table declares it.
  pub const fn get_encoding_label(self) -> &'static str {
    match self {
      Self::French | Self::German | Self::Italian | Self::Spanish => "windows-1252",
      Self::Polish => "windows-1250",
      Self::All | Self::English | Self::Russian | Self::Ukrainian => "windows-1251",
    }
  }

  /// The single language a code names.
  ///
  /// # Errors
  ///
  /// Returns a parsing error for a code that names no language, and an unknown-language error for `all`.
  pub fn from_str_single(language: &str) -> XrfResult<Self> {
    Self::from_str(language)
      .map_err(XrfError::new_parsing_error)?
      .into_single()
  }

  /// This language, refused when it stands for every language at once.
  ///
  /// # Errors
  ///
  /// Returns an unknown-language error for [`Self::All`].
  pub fn into_single(self) -> XrfResult<Self> {
    match self {
      Self::All => Err(XrfError::new_unknown_language_error(String::from(
        "Unexpected language 'all' provided",
      ))),
      language => Ok(language),
    }
  }

  /// Read the language off the `rus` of a `text/rus/st_dialogs.xml`, which is how gamedata carries it.
  pub(crate) fn from_directory_name(directory_name: &str) -> Option<Self> {
    Self::from_str_single(directory_name).ok()
  }
}
