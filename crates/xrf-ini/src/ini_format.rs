use crate::ini_encoding::IniEncoding;
use crate::ini_repeated_key::IniRepeatedKey;

/// How a file is read and written.
///
/// Beyond `[section]` headers and `key = value` fields, every convention is a field here, so nothing is assumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IniFormat {
  pub encoding: IniEncoding,
  /// Written after every line. Reading accepts `\n` and `\r\n` alike.
  pub line_end: &'static str,
  /// Spaces written before every field.
  pub field_indent: usize,
  /// Width a key and a value are padded to with spaces; the line is trimmed at the right after.
  pub column_width: usize,
  /// A line written after every section's fields.
  pub section_end: Option<&'static str>,
  /// Whether a value whose double quotes are left open continues on the next lines until they close.
  pub is_multiline_quoted: bool,
  /// What a read does with a key set twice in one section.
  pub repeated_key: IniRepeatedKey,
  /// Whether sections, and each section's fields, are written ordered by name as `strcmp` orders their bytes.
  pub is_sorted: bool,
}

impl IniFormat {
  /// A plain INI file: UTF-8, `\n`, `key = value` in the order given, one line per value.
  pub const PLAIN: Self = Self {
    encoding: IniEncoding::Utf8,
    line_end: "\n",
    field_indent: 0,
    column_width: 0,
    section_end: None,
    is_multiline_quoted: false,
    repeated_key: IniRepeatedKey::Refused,
    is_sorted: false,
  };

  /// What the engine's and the SDK's `CInifile::save_as` in `xr_ini.cpp` writes.
  ///
  /// Windows-1251, `\r\n`, fields as `%8s%-32s = %-32s` trimmed at the right, a one-space line after each section, and
  /// `strcmp` order. Quoted values may span lines and a repeated key keeps its last value, as in the engine. Values are
  /// written as set, without the engine's reformatting; the README lists where reading hand-edited files differs.
  pub const XRAY_SAVE_AS: Self = Self {
    encoding: IniEncoding::Windows1251,
    line_end: "\r\n",
    field_indent: 8,
    column_width: 32,
    section_end: Some(" "),
    is_multiline_quoted: true,
    repeated_key: IniRepeatedKey::LastValueKept,
    is_sorted: true,
  };
}

impl Default for IniFormat {
  fn default() -> Self {
    Self::PLAIN
  }
}
