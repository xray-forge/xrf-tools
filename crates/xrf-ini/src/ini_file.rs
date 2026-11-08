use std::collections::HashSet;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::ini_field::IniField;
use crate::ini_format::IniFormat;
use crate::ini_override::IniOverride;
use crate::ini_repeated_key::IniRepeatedKey;
use crate::ini_section::IniSection;

/// An INI file: sections and their fields, in the order they were read or added.
///
/// Every method without a format reads or writes [`IniFormat::PLAIN`]; the `_as` methods take the format to use, such
/// as [`IniFormat::XRAY_SAVE_AS`] for a file the engine or the SDK saved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IniFile {
  pub sections: Vec<IniSection>,
  /// Values a read dropped because a later line set the same key again, under a format that keeps the last value;
  /// a write leaves them out, as the engine does.
  pub overridden: Vec<IniOverride>,
}

impl IniFile {
  pub fn section(&self, name: &str) -> Option<&IniSection> {
    self.sections.iter().find(|section| section.name == name)
  }

  /// A section the file must hold.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section when it is missing.
  pub fn section_required(&self, name: &str) -> XrfResult<&IniSection> {
    self
      .section(name)
      .ok_or_else(|| XrfError::new_parsing_error(format!("File has no section [{name}]")))
  }

  /// The section of that name, added after the others when the file has none.
  pub fn section_mut(&mut self, name: &str) -> &mut IniSection {
    let index: usize = match self.sections.iter().position(|section| section.name == name) {
      Some(index) => index,
      None => {
        self.sections.push(IniSection::new(name));

        self.sections.len() - 1
      }
    };

    &mut self.sections[index]
  }

  pub fn get(&self, section: &str, key: &str) -> Option<&str> {
    self.section(section)?.get(key)
  }

  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is not a plain INI file.
  pub fn read_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_path_as(path, &IniFormat::PLAIN)
  }

  /// # Errors
  ///
  /// Returns an error when the file cannot be read or does not read in `format`.
  pub fn read_from_path_as<P: AsRef<Path>>(path: &P, format: &IniFormat) -> XrfResult<Self> {
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "INI file was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?;

    Self::read_from_bytes_as(&bytes, format)
  }

  /// # Errors
  ///
  /// Returns an error when the bytes are not a plain INI file.
  pub fn read_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    Self::read_from_bytes_as(bytes, &IniFormat::PLAIN)
  }

  /// # Errors
  ///
  /// Returns an error when the bytes are not valid in the format's encoding, or not text [`Self::read_from_str_as`]
  /// reads.
  pub fn read_from_bytes_as(bytes: &[u8], format: &IniFormat) -> XrfResult<Self> {
    Self::read_from_str_as(&format.encoding.decode(bytes)?, format)
  }

  /// # Errors
  ///
  /// Returns an error as [`Self::read_from_str_as`] does.
  pub fn read_from_str(text: &str) -> XrfResult<Self> {
    Self::read_from_str_as(text, &IniFormat::PLAIN)
  }

  /// Reads text: a `[name]` line opens a section, a `key = value` line is a field (a line with no `=` is a key with an
  /// empty value), and blank lines and lines starting with `;` are skipped. Keys and values are trimmed at both ends.
  /// Where the format reads multi-line quoted values, a value leaving a double quote open takes the next lines, joined
  /// by the format's line end, until it closes. A key set twice in a section is refused, or keeps its last value in its
  /// first place with the dropped one named in [`Self::overridden`], as the format says.
  ///
  /// # Errors
  ///
  /// Returns an error, with the line, for a field before any section, a header with anything but a `;` comment after
  /// its `]` (parents are an LTX feature, read by `xrf-ltx`), a field with no key, a key set twice in a section where
  /// the format refuses it, a section declared twice, or a quote still open at the end of the text.
  pub fn read_from_str_as(text: &str, format: &IniFormat) -> XrfResult<Self> {
    let mut file: Self = Self::default();
    let mut names: HashSet<String> = HashSet::new();
    let mut lines = text
      .split('\n')
      .map(|line| line.strip_suffix('\r').unwrap_or(line))
      .enumerate();

    while let Some((index, line)) = lines.next() {
      let number: usize = index + 1;
      let content: &str = line.trim();

      if content.is_empty() || content.starts_with(';') {
        continue;
      }

      if let Some(header) = content.strip_prefix('[') {
        let (name, rest) = header
          .split_once(']')
          .ok_or_else(|| Self::line_error(number, "the section header has no ']'"))?;
        let rest: &str = rest.trim();

        if !rest.is_empty() && !rest.starts_with(';') {
          return Err(Self::line_error(
            number,
            &format!("section [{name}] is followed by '{rest}'"),
          ));
        }

        if !names.insert(name.to_owned()) {
          return Err(Self::line_error(number, &format!("section [{name}] is declared twice")));
        }

        file.sections.push(IniSection::new(name));

        continue;
      }

      let section: &mut IniSection = file
        .sections
        .last_mut()
        .ok_or_else(|| Self::line_error(number, "a field stands before any section"))?;
      let (key, value) = content.split_once('=').unwrap_or((content, ""));
      let key: &str = key.trim();
      let mut value: String = value.trim().to_owned();

      if key.is_empty() {
        return Err(Self::line_error(number, "a field has no key"));
      }

      if format.is_multiline_quoted {
        while IniField::is_quote_open(&value) {
          let (_, next) = lines
            .next()
            .ok_or_else(|| Self::line_error(number, &format!("field '{key}' leaves a quote open")))?;

          value.push_str(format.line_end);
          value.push_str(next);
        }

        value.truncate(value.trim_end().len());
      }

      let Some(earlier) = section.fields.iter_mut().find(|field| field.key == key) else {
        section.fields.push(IniField::new(key, value));

        continue;
      };

      if format.repeated_key == IniRepeatedKey::Refused {
        return Err(Self::line_error(
          number,
          &format!("field '{key}' is set twice in section [{}]", section.name),
        ));
      }

      file.overridden.push(IniOverride {
        section: section.name.clone(),
        key: key.to_owned(),
        value: std::mem::replace(&mut earlier.value, value),
        line: number,
      });
    }

    Ok(file)
  }

  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_path_as`] does.
  pub fn write_to_path<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    self.write_to_path_as(path, &IniFormat::PLAIN)
  }

  /// Writes the file through a staged file, creating the directories leading to it.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_bytes_as`] does, or when the file cannot be written.
  pub fn write_to_path_as<P: AsRef<Path>>(&self, path: &P, format: &IniFormat) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      path.as_ref(),
      &self.write_to_bytes_as(format)?,
    )?)
  }

  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_string_as`] does.
  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    self.write_to_bytes_as(&IniFormat::PLAIN)
  }

  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_string_as`] does, or when the text has a character the format's encoding
  /// has no byte for.
  pub fn write_to_bytes_as(&self, format: &IniFormat) -> XrfResult<Vec<u8>> {
    format.encoding.encode(self.write_to_string_as(format)?)
  }

  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_string_as`] does.
  pub fn write_to_string(&self) -> XrfResult<String> {
    self.write_to_string_as(&IniFormat::PLAIN)
  }

  /// Writes the text: every section's `[name]`, then each field as the format's indent, the key padded to its column
  /// width, ` = ` and the value padded the same, trimmed at the right, then the format's section end line, each line
  /// ended with the format's line end. A sorted format writes sections and fields in `strcmp` order.
  ///
  /// # Errors
  ///
  /// Returns an error for anything that would not read back as itself: an empty section name or one holding a
  /// bracket or line break, a key that is empty, starts with `;` or `[`, or holds `=` or a line break, a value with
  /// a line break where the format has none or outside a quoted string, or one that starts or ends with whitespace.
  pub fn write_to_string_as(&self, format: &IniFormat) -> XrfResult<String> {
    let mut text: String = String::new();

    for section in Self::ordered(&self.sections, format, |section| &section.name) {
      Self::check_section_name(&section.name)?;

      text.push('[');
      text.push_str(&section.name);
      text.push(']');
      text.push_str(format.line_end);

      for field in Self::ordered(&section.fields, format, |field| &field.key) {
        Self::check_field(&section.name, field, format)?;

        let line: String = format!(
          "{:indent$}{:<width$} = {:<width$}",
          "",
          field.key,
          field.value,
          indent = format.field_indent,
          width = format.column_width
        );

        text.push_str(line.trim_end_matches([' ', '\t']));
        text.push_str(format.line_end);
      }

      if let Some(section_end) = format.section_end {
        text.push_str(section_end);
        text.push_str(format.line_end);
      }
    }

    Ok(text)
  }

  /// The items in the order the format writes them: as given, or sorted by name as `strcmp` sorts the encoded bytes.
  fn ordered<'a, T>(items: &'a [T], format: &IniFormat, name: impl Fn(&T) -> &String) -> Vec<&'a T> {
    let mut ordered: Vec<&T> = items.iter().collect();

    if format.is_sorted {
      ordered.sort_by(|left, right| format.encoding.compare(name(left), name(right)));
    }

    ordered
  }

  fn check_section_name(name: &str) -> XrfResult {
    if name.is_empty() || name.contains(['[', ']', '\r', '\n']) {
      return Err(XrfError::new_invalid_error(format!(
        "Section name '{name}' would not read back as itself"
      )));
    }

    Ok(())
  }

  fn check_field(section: &str, field: &IniField, format: &IniFormat) -> XrfResult {
    let key: &str = &field.key;

    if key.is_empty() || key != key.trim() || key.contains(['=', '\r', '\n']) || key.starts_with([';', '[']) {
      return Err(XrfError::new_invalid_error(format!(
        "Key '{key}' in section [{section}] would not read back as itself"
      )));
    }

    if field.value != field.value.trim() {
      return Err(XrfError::new_invalid_error(format!(
        "Value of [{section}] {key} starts or ends with whitespace, which a read trims"
      )));
    }

    let mut is_quoted: bool = false;

    for character in field.value.chars() {
      match character {
        '"' => is_quoted = !is_quoted,
        '\r' | '\n' if !format.is_multiline_quoted || !is_quoted => {
          return Err(XrfError::new_invalid_error(format!(
            "Value of [{section}] {key} breaks a line outside a quoted string the format reads across lines"
          )));
        }
        _ => {}
      }
    }

    Ok(())
  }

  fn line_error(number: usize, message: &str) -> XrfError {
    XrfError::new_parsing_error(format!("Line {number}: {message}"))
  }
}
