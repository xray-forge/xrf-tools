use std::fs;
use std::io::Write;
use std::path::Path;

use xrf_error::XrfResult;
use xrf_utils::{StagedFile, encode_string_into_w1251_bytes, read_as_string_from_w1251_encoded};

use crate::binding::LtxWrite;
use crate::ltx::{Ltx, SectionSetter};
use crate::syntax::{DEFAULT_KV_SEPARATOR, EMPTY_VALUE_KV_SEPARATOR, LTX_LINE_SEPARATOR, ROOT_SECTION};

impl LtxWrite for Ltx {
  type SectionWriter<'a> = SectionSetter<'a>;

  fn with_section<'a>(&'a mut self, section: &'a str) -> SectionSetter<'a> {
    SectionSetter::new(self, String::from(section))
  }
}

impl Ltx {
  /// Format single LTX file by provided path
  pub(crate) fn format_file<P: AsRef<Path>>(filename: P, write: bool) -> XrfResult<bool> {
    let formatted: String = Self::format_from_file(&filename)?;
    let existing: String =
      read_as_string_from_w1251_encoded(&mut fs::OpenOptions::new().read(true).open(filename.as_ref())?)?;

    if existing == formatted {
      Ok(false)
    } else {
      if write {
        // The read path above decodes Windows-1251, so the rewrite has to go back in that encoding. Staged, so a
        // failed write leaves the source as it was.
        StagedFile::write(filename.as_ref(), &encode_string_into_w1251_bytes(formatted)?)?;
      }

      Ok(true)
    }
  }

  /// Write to a file, making the directories above it: encoded whole first and staged beside it, so a value the
  /// encoding refuses or a failed write leaves whatever was there before.
  pub fn write_to_path<P: AsRef<Path>>(&self, filename: P) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      filename.as_ref(),
      &self.write_to_bytes()?,
    )?)
  }

  /// Write to a writer, in the Windows-1251 encoding the read path decodes.
  pub fn write_to<W: Write>(&self, writer: &mut W) -> XrfResult {
    writer.write_all(&self.write_to_bytes()?)?;

    Ok(())
  }

  /// The file's bytes, in the Windows-1251 encoding the read path decodes.
  ///
  /// Rendered as text first and encoded whole, because the text is Rust's UTF-8, which the next read would decode as
  /// something else. ASCII text, as an exported spawn or particle library is, encodes to itself, so its buffer is
  /// handed back as the bytes rather than copied.
  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    Ok(encode_string_into_w1251_bytes(self.render())?)
  }

  /// The text of the file, measured by a first pass so a document of tens of megabytes is laid down once rather than
  /// copied each time a growing buffer doubles.
  fn render(&self) -> String {
    let mut length: usize = 0;

    self.render_pieces(|piece| length += piece.len());

    let mut rendered: String = String::with_capacity(length);

    self.render_pieces(|piece| rendered.push_str(piece));

    rendered
  }

  /// Hands `emit` the file's text piece by piece, in order: the one place its layout is written down.
  fn render_pieces(&self, mut emit: impl FnMut(&str)) {
    let mut firstline: bool = true;

    for include in &self.includes {
      firstline = false;
      emit("#include \"");
      emit(include);
      emit("\"");
      emit(LTX_LINE_SEPARATOR);
    }

    for (section, props) in &self.sections {
      // If root section with data or generic section.
      if section != ROOT_SECTION || !props.data.is_empty() {
        if firstline {
          firstline = false;
        } else {
          // Write an empty line between sections
          emit(LTX_LINE_SEPARATOR);
        }
      }

      if section != ROOT_SECTION {
        emit("[");
        emit(section);
        emit("]");

        for (index, parent) in props.inherited.iter().enumerate() {
          emit(if index == 0 { ":" } else { ", " });
          emit(parent);
        }

        emit(LTX_LINE_SEPARATOR);
      }

      for (key, value) in props {
        emit(key);
        emit(if value.is_empty() {
          EMPTY_VALUE_KV_SEPARATOR
        } else {
          DEFAULT_KV_SEPARATOR
        });
        emit(value);
        emit(LTX_LINE_SEPARATOR);
      }
    }
  }
}

#[cfg(test)]
mod test {
  use xrf_utils::{decode_w1251_bytes_to_string, encode_string_to_w1251_bytes};

  use crate::binding::LtxSectionWrite;
  use crate::ltx::Ltx;
  use crate::syntax::{LTX_LINE_SEPARATOR, ROOT_SECTION};

  #[test]
  fn preserve_order_write() {
    let input = r"
x2 = n2
x1 = n2
x3 = n2
[s]
x2 = n2
xb = n2
a3 = n3
";
    let ltx: Ltx = Ltx::read_from_str(input).unwrap();
    let mut buf = vec![];
    ltx.write_to(&mut buf).unwrap();
    let new_data = Ltx::read_from_str(&String::from_utf8(buf).unwrap()).unwrap();

    let sec0 = new_data.root_section().expect("root fields to be declared");
    let keys0: Vec<&str> = sec0.iter().map(|(k, _)| k).collect();
    assert_eq!(keys0, vec!["x2", "x1", "x3"]);

    let sec1 = new_data.section("s").unwrap();
    let keys1: Vec<&str> = sec1.iter().map(|(k, _)| k).collect();
    assert_eq!(keys1, vec!["x2", "xb", "a3"]);
  }

  #[test]
  fn write_new() {
    use std::str;

    let ltx: Ltx = Ltx::new();

    let mut buf = Vec::new();
    ltx.write_to(&mut buf).unwrap();

    assert_eq!("", str::from_utf8(&buf).unwrap());
  }

  #[test]
  fn write_line_separator() {
    use std::str;

    let mut ltx: Ltx = Ltx::new();

    ltx.with_section("Section1").set("Key1", "Value").set("Key2", "Value");
    ltx.with_section("Section2").set("Key1", "Value").set("Key2", "Value");

    {
      let mut buf: Vec<u8> = Vec::new();
      ltx.write_to(&mut buf).unwrap();

      assert_eq!(
        "[Section1]\r\nKey1 = Value\r\nKey2 = Value\r\n\r\n[Section2]\r\nKey1 = Value\r\nKey2 = Value\r\n",
        str::from_utf8(&buf).unwrap()
      );
    }
  }

  /// Includes, parents and root fields each take their own branch of the layout, and the text is sized before it is
  /// written, so the exact bytes are pinned here.
  #[test]
  fn writes_includes_parents_and_root_fields_in_their_places() {
    let mut ltx: Ltx = Ltx::new();

    ltx.include(String::from("first.ltx"));
    ltx.include(String::from("second.ltx"));
    ltx.with_root_section().set("root", 1);
    ltx.with_section("child").set("key", "value").set("empty", "");
    ltx.section_mut("child").expect("the section").inherit("first_parent");
    ltx.section_mut("child").expect("the section").inherit("second_parent");
    ltx.with_section("only_parent").set("x", 2.5);
    ltx.section_mut("only_parent").expect("the section").inherit("base");

    let bytes: Vec<u8> = ltx.write_to_bytes().unwrap();

    assert_eq!(
      String::from_utf8(bytes.clone()).unwrap(),
      "#include \"first.ltx\"\r\n#include \"second.ltx\"\r\n\r\nroot = 1\r\n\r\n\
       [child]:first_parent, second_parent\r\nkey = value\r\nempty =\r\n\r\n[only_parent]:base\r\nx = 2.5\r\n"
    );
    assert_eq!(
      bytes.len(),
      bytes.capacity(),
      "the text to be sized exactly before it is written"
    );
  }

  #[test]
  fn write_empty_values_without_trailing_spaces() {
    let mut ltx: Ltx = Ltx::new();

    ltx.with_section(ROOT_SECTION).set("EmptyRoot", "");
    ltx
      .with_section("Section")
      .set("Empty", "")
      .set("Value", "text")
      .set("Whitespace", " \t ");

    let mut output: Vec<u8> = Vec::new();
    ltx.write_to(&mut output).unwrap();

    let rendered: String = String::from_utf8(output).unwrap();

    assert_eq!(
      rendered,
      "EmptyRoot =\r\n\r\n[Section]\r\nEmpty =\r\nValue = text\r\nWhitespace =  \t \r\n"
    );

    let round_tripped: Ltx = Ltx::read_from_str(&rendered).unwrap();

    assert_eq!(round_tripped.root_section().unwrap().get("EmptyRoot"), Some(""));
    assert_eq!(round_tripped.section("Section").unwrap().get("Empty"), Some(""));
    assert_eq!(round_tripped.section("Section").unwrap().get("Value"), Some("text"));
  }

  #[test]
  fn write_kv_separator() {
    use std::str;

    let mut ltx: Ltx = Ltx::new();

    ltx.with_section(ROOT_SECTION).set("Key1", "Value").set("Key2", "Value");
    ltx.with_section("Section1").set("Key1", "Value").set("Key2", "Value");
    ltx.with_section("Section2").set("Key1", "Value").set("Key2", "Value");

    let mut buf: Vec<u8> = Vec::new();
    ltx.write_to(&mut buf).unwrap();

    assert_eq!(
      str::from_utf8(&buf).unwrap(),
      "Key1 = Value\r\nKey2 = Value\r\n\r\n[Section1]\r\nKey1 = Value\r\nKey2 = Value\r\n\r\n[Section2]\r\nKey1 = Value\r\nKey2 = Value\r\n",
    );
  }

  /// Non-ASCII values survive the write, which emits Windows-1251 rather than UTF-8 so that the read path decodes
  /// back to the same text. Characters outside that encoding are refused rather than silently mangled.
  #[test]
  fn write_non_ascii_as_windows_1251() {
    let input: String = format!("some-key = мова{LTX_LINE_SEPARATOR}");
    let ltx: Ltx = Ltx::read_from_str(&input).unwrap();

    let mut output: Vec<u8> = Vec::new();
    ltx.write_to(&mut output).unwrap();

    assert_eq!(output, encode_string_to_w1251_bytes(&input).unwrap());
    assert_eq!(decode_w1251_bytes_to_string(&output).unwrap(), input);
  }

  #[test]
  fn refuse_values_outside_windows_1251() {
    let ltx: Ltx = Ltx::read_from_str(&format!("some-key = åäö{LTX_LINE_SEPARATOR}")).unwrap();

    assert!(ltx.write_to(&mut Vec::new()).is_err());
  }
}
