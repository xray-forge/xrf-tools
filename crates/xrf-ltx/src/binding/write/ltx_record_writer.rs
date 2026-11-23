use std::fmt::{Display, Write};
use std::hash::BuildHasher;
use std::path::Path;

use xrf_collections::FastBuildHasher;
use xrf_collections::hashbrown::HashTable;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, encode_string_into_w1251_bytes};

use crate::binding::{LtxFieldIndex, LtxRecordSection, LtxWrite};
use crate::syntax::{
  DEFAULT_KV_SEPARATOR, EMPTY_VALUE_KV_SEPARATOR, LTX_LINE_SEPARATOR, ROOT_SECTION, leaves_quote_open,
  new_open_quote_error,
};

/// Writes a file of exported records as its text while they are exported, without building a config of them.
#[derive(Default)]
pub struct LtxRecordWriter {
  text: String,
  /// Where each finished section's name lies in the text.
  finished: HashTable<TextSpan>,
  /// Where the open section's name lies in the text, once a field opened one.
  open: Option<TextSpan>,
  /// The open section's fields, in written order.
  fields: Vec<FieldLine>,
  /// The open section's fields by key.
  field_keys: LtxFieldIndex,
  /// The first section written to again after another had started.
  reopened: Option<String>,
  /// Whether the root section was opened after another, where its headerless fields would read back as that one's.
  is_root_late: bool,
  /// The first section and key given a value that leaves a quote open, which no reader takes back.
  open_quote: Option<(String, String)>,
}

/// A run of the writer's text.
#[derive(Clone, Copy)]
struct TextSpan {
  start: usize,
  end: usize,
}

/// A field's line in the writer's text.
#[derive(Clone, Copy)]
struct FieldLine {
  key: TextSpan,
  /// Where the line ends, before its line break.
  end: usize,
}

impl TextSpan {
  fn of(self, text: &str) -> &str {
    &text[self.start..self.end]
  }

  /// Whether this is the root section's name, which takes no text as the root has no header.
  fn is_root(self) -> bool {
    self.start == self.end
  }
}

impl LtxRecordWriter {
  pub fn new() -> Self {
    Self::default()
  }

  /// Writes the file through a staged file, creating its directories, as [`crate::Ltx::write_to_path`] does.
  ///
  /// # Errors
  ///
  /// Returns an error for what [`Self::into_bytes`] refuses, or when the file cannot be written.
  pub fn write_to_path<P: AsRef<Path>>(self, path: P) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      path.as_ref(),
      &self.into_bytes()?,
    )?)
  }

  /// The file's bytes, in the Windows-1251 encoding [`crate::Ltx::write_to_bytes`] writes.
  ///
  /// # Errors
  ///
  /// Returns an error when a section, the root included, was written to after another started, a value in a section
  /// leaves a quote open, or a value has no Windows-1251 encoding.
  pub fn into_bytes(mut self) -> XrfResult<Vec<u8>> {
    self.finish_section();

    if let Some((section, key)) = self.open_quote {
      return Err(new_open_quote_error(&section, &key));
    }

    if let Some(section) = self.reopened {
      return Err(XrfError::new_convert_error(format!(
        "Section '{section}' is written to again after another section started, which a record file cannot hold"
      )));
    }

    if self.is_root_late {
      return Err(XrfError::new_convert_error(
        "The root section is written to after another section started; its fields would read back as that section's",
      ));
    }

    Ok(encode_string_into_w1251_bytes(self.text)?)
  }

  /// Opens `name`, finishing the section before it; false when `name` was finished, or is a late root section.
  pub(crate) fn open_section(&mut self, name: &str) -> bool {
    if self.open.is_some_and(|open| open.of(&self.text) == name) {
      return true;
    }

    if name == ROOT_SECTION && (self.open.is_some() || !self.finished.is_empty()) {
      self.is_root_late = true;

      return false;
    }

    let text: &str = &self.text;

    if self
      .finished
      .find(Self::hash(name), |finished| finished.of(text) == name)
      .is_some()
    {
      self.reopened.get_or_insert_with(|| String::from(name));

      return false;
    }

    self.finish_section();

    // A blank line between sections, as a config writes them.
    if !self.text.is_empty() {
      self.text.push_str(LTX_LINE_SEPARATOR);
    }

    let open: TextSpan = if name == ROOT_SECTION {
      // The root section's fields stand at the top with no header, so its name takes no text.
      TextSpan {
        start: self.text.len(),
        end: self.text.len(),
      }
    } else {
      self.text.push('[');

      let start: usize = self.text.len();

      self.text.push_str(name);

      let end: usize = self.text.len();

      self.text.push(']');
      self.text.push_str(LTX_LINE_SEPARATOR);

      TextSpan { start, end }
    };

    self.open = Some(open);

    true
  }

  pub(crate) fn set_field<V: Display>(&mut self, key: &str, value: V) {
    match self.find_field(key) {
      Some(index) => self.replace_value(index, value),
      None => self.push_field(key, value),
    }
  }

  fn finish_section(&mut self) {
    let Some(open) = self.open.take() else {
      return;
    };
    let text: &str = &self.text;

    // Each field's value as it was last set; the root section's stand before every header, where a reader takes an
    // open quote as one line.
    if self.open_quote.is_none()
      && !open.is_root()
      && let Some(field) = self
        .fields
        .iter()
        .find(|field| leaves_quote_open(&text[field.key.end..field.end]))
    {
      self.open_quote = Some((String::from(open.of(text)), String::from(field.key.of(text))));
    }

    self
      .finished
      .insert_unique(Self::hash(open.of(text)), open, |finished| {
        Self::hash(finished.of(text))
      });

    self.field_keys.clear();
    self.fields.clear();
  }

  fn find_field(&self, key: &str) -> Option<usize> {
    let (text, fields): (&str, &[FieldLine]) = (&self.text, &self.fields);

    self.field_keys.find(key, |index| fields[index].key.of(text))
  }

  fn push_field<V: Display>(&mut self, key: &str, value: V) {
    let start: usize = self.text.len();

    self.text.push_str(key);

    let key: TextSpan = TextSpan {
      start,
      end: self.text.len(),
    };

    Self::push_value(&mut self.text, value);
    self.fields.push(FieldLine {
      key,
      end: self.text.len(),
    });
    self.text.push_str(LTX_LINE_SEPARATOR);

    let (text, fields): (&str, &[FieldLine]) = (&self.text, &self.fields);

    self.field_keys.push(|index| fields[index].key.of(text));
  }

  /// Overwrites a field's value in place, shifting the lines after it.
  fn replace_value<V: Display>(&mut self, index: usize, value: V) {
    let mut written: String = String::new();

    Self::push_value(&mut written, value);

    let field: FieldLine = self.fields[index];
    let replaced: usize = field.end - field.key.end;
    let moved = |position: usize| position - replaced + written.len();

    self.text.replace_range(field.key.end..field.end, &written);
    self.fields[index].end = field.key.end + written.len();

    for later in &mut self.fields[index + 1..] {
      later.key.start = moved(later.key.start);
      later.key.end = moved(later.key.end);
      later.end = moved(later.end);
    }
  }

  /// The separator and the value, as a config writes them after a key.
  fn push_value<V: Display>(text: &mut String, value: V) {
    text.push_str(DEFAULT_KV_SEPARATOR);

    let start: usize = text.len();

    // Writing to memory fails only when the value's own `Display` reports an error, which `to_string` panics on.
    write!(text, "{value}").expect("a Display implementation returned an error unexpectedly");

    // The value is known only once written, so an empty one takes back the separator it followed.
    if text.len() == start {
      text.truncate(start - DEFAULT_KV_SEPARATOR.len());
      text.push_str(EMPTY_VALUE_KV_SEPARATOR);
    }
  }

  fn hash(text: &str) -> u64 {
    FastBuildHasher.hash_one(text)
  }
}

impl LtxWrite for LtxRecordWriter {
  type SectionWriter<'a> = LtxRecordSection<'a>;

  fn with_section<'a>(&'a mut self, section: &'a str) -> LtxRecordSection<'a> {
    LtxRecordSection::new(self, section)
  }
}

#[cfg(test)]
mod test {
  use xrf_error::XrfResult;

  use crate::binding::{LtxRecordWriter, LtxSectionWrite, LtxWrite};
  use crate::ltx::Ltx;
  use crate::syntax::ROOT_SECTION;

  fn export_records<W: LtxWrite>(writer: &mut W) {
    writer
      .with_section("0")
      .set("position", "1,2,3")
      .set("distance", 2.5)
      .set("level_vertex_id", 40_u32);
    writer.with_section("1").set("position", "4,5,6").set("empty", "");

    // A nested export writing into the section its caller opened, and a key the caller wrote already.
    writer.with_section("1").set("position", "7,8,9").set("tail", "x");

    writer
      .with_section("emptied")
      .set("value", "text")
      .set("value", "")
      .set("filled", "")
      .set("filled", 10)
      .set("after", "kept");

    // Enough fields that a repeated key is found by its hash rather than by a scan.
    {
      let mut many = writer.with_section("many");

      for index in 0..20 {
        many.set(format!("key_{index}"), index);
      }

      many
        .set("key_3", "again")
        .set("key_15", "")
        .set("key_19", "last")
        .set("key_20", "new");
    }

    writer.with_section("absent").set_optional("never", None::<u32>);
    writer
      .with_section("optional")
      .set_optional("some", Some("value"))
      .set_optional("none", None::<u8>);
    writer
      .with_section("text")
      .set("ukrainian", "мова")
      .set("spaced", " a b ");
  }

  // A config writes root fields first wherever they were set; a record writer, writing as it goes, cannot.
  #[test]
  fn keeps_root_fields_set_late_out_of_the_section_before_them() -> XrfResult {
    fn write_root_late<W: LtxWrite>(writer: &mut W) {
      writer.with_section("first").set("a", 1);
      writer.with_section(ROOT_SECTION).set("root", "late");
    }

    let mut ltx: Ltx = Ltx::new();
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    write_root_late(&mut ltx);
    write_root_late(&mut writer);

    let reread: Ltx = Ltx::read_from_str(&String::from_utf8(ltx.write_to_bytes()?).expect("ASCII text"))?;

    assert_eq!(reread.get_from(ROOT_SECTION, "root"), Some("late"));
    assert_eq!(reread.get_from("first", "root"), None);
    assert!(writer.into_bytes().is_err());

    Ok(())
  }

  #[test]
  fn writes_what_a_config_writes_for_the_same_records() -> XrfResult {
    let mut ltx: Ltx = Ltx::new();
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    export_records(&mut ltx);
    export_records(&mut writer);

    let written: Vec<u8> = writer.into_bytes()?;

    assert_eq!(written, ltx.write_to_bytes()?);
    assert!(
      !String::from_utf8_lossy(&written).contains("[absent]"),
      "a section that took no field to leave no header"
    );

    Ok(())
  }

  fn export_root_first<W: LtxWrite>(writer: &mut W) {
    writer.with_section(ROOT_SECTION).set("root", "").set("other", 1);
    writer.with_section("section").set("key", "value");
  }

  #[test]
  fn writes_the_root_section_first_without_a_header() -> XrfResult {
    let mut ltx: Ltx = Ltx::new();
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    export_root_first(&mut ltx);
    export_root_first(&mut writer);

    let expected: &[u8] = b"root =\r\nother = 1\r\n\r\n[section]\r\nkey = value\r\n";

    assert_eq!(writer.into_bytes()?, expected);
    assert_eq!(ltx.write_to_bytes()?, expected);

    Ok(())
  }

  #[test]
  fn refuses_a_section_written_to_after_another_started() {
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    writer.with_section("first").set("a", 1);
    writer.with_section("second").set("b", 2);
    writer.with_section("first").set("c", 3);
    writer.with_section("second").set("d", 4);
    writer.with_section("third").set("e", 5);

    assert_eq!(
      writer
        .into_bytes()
        .expect_err("the reopened section to be refused")
        .to_string(),
      "Convert error: Section 'first' is written to again after another section started, which a record file cannot \
       hold"
    );
  }

  #[test]
  fn refuses_the_root_section_written_to_after_another_started() {
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    writer.with_section(ROOT_SECTION).set("a", 1);
    writer.with_section("section").set("b", 2);
    writer.with_section(ROOT_SECTION).set("c", 3);

    assert!(writer.into_bytes().is_err());
  }

  #[test]
  fn refuses_a_value_leaving_a_quote_open_naming_its_section_and_key() {
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    writer.with_section(ROOT_SECTION).set("root", "\"open");
    writer.with_section("box").set("closed", "\"a\"").set("x", "\"a");
    writer.with_section("later").set("y", "\"b");

    assert_eq!(
      writer.into_bytes().expect_err("an open quote").to_string(),
      "Convert error: Value of 'x' in section 'box' leaves a quote open, which no reader takes back"
    );

    let mut replaced: LtxRecordWriter = LtxRecordWriter::new();

    replaced.with_section("box").set("x", "\"a").set("x", "\"a\"");

    assert!(
      replaced.into_bytes().is_ok(),
      "a value is judged as it was last set, as a config judges it"
    );
  }

  #[test]
  fn refuses_a_value_outside_windows_1251() {
    let mut writer: LtxRecordWriter = LtxRecordWriter::new();

    writer.with_section("section").set("key", "åäö");

    assert!(writer.into_bytes().is_err());
  }

  #[test]
  fn writes_nothing_for_no_records() -> XrfResult {
    assert_eq!(LtxRecordWriter::new().into_bytes()?, Vec::<u8>::new());

    Ok(())
  }
}
