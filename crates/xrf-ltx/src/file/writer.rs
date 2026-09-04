use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use xrf_error::XrfResult;
use xrf_utils::{encode_string_to_w1251_bytes, read_as_string_from_w1251_encoded};

use crate::Ltx;
use crate::file::file_configuration::constants::ROOT_SECTION;
use crate::file::file_configuration::line_separator::{DEFAULT_KV_SEPARATOR, LineSeparator};

impl Ltx {
  /// Format single LTX file by provided path
  pub fn format_file<P: AsRef<Path>>(filename: P, write: bool) -> XrfResult<bool> {
    let formatted: String = Ltx::format_from_file(&filename)?;
    let existing: String =
      read_as_string_from_w1251_encoded(&mut fs::OpenOptions::new().read(true).open(filename.as_ref())?)?;

    if existing == formatted {
      Ok(false)
    } else {
      if write {
        // The read path above decodes Windows-1251, so the rewrite has to go back in that encoding.
        fs::write(&filename, encode_string_to_w1251_bytes(&formatted)?)?;
      }

      Ok(true)
    }
  }

  /// Write to a file
  pub fn write_to_path<P: AsRef<Path>>(&self, filename: P) -> XrfResult {
    self.write_to(
      &mut fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename.as_ref())?,
    )
  }

  /// Write to a writer, in the Windows-1251 encoding the read path decodes.
  ///
  /// Rendered into a buffer first because `write!` of a Rust string emits UTF-8, which the next read would decode as
  /// something else. Encoding per `write!` is not an option: a format piece can end mid-character.
  pub fn write_to<W: Write>(&self, writer: &mut W) -> XrfResult {
    let mut rendered: Vec<u8> = Vec::new();

    self.render_to(&mut rendered)?;

    writer.write_all(&encode_string_to_w1251_bytes(
      // Every piece written below came from a Rust string, so the buffer is valid UTF-8.
      &String::from_utf8(rendered).expect("Rendered LTX to be valid UTF-8"),
    )?)?;

    Ok(())
  }

  /// Buffered here rather than at the call sites. Every line below is a `write!`, and `write_fmt` issues one
  /// `write_all` per format piece, so writing straight to a `File` cost four syscalls a line — two orders of magnitude
  /// slower than this on an export the size of the particles library. Flushed explicitly so a failing flush is returned
  /// instead of being swallowed when the buffer drops.
  fn render_to<W: Write>(&self, writer: &mut W) -> XrfResult {
    let mut writer: BufWriter<&mut W> = BufWriter::new(writer);
    let mut firstline: bool = true;

    // Write include statements.
    if !self.includes.is_empty() {
      firstline = false;

      for include in &self.includes {
        write!(writer, "#include \"{}\"{}", include, LineSeparator::CRLF)?;
      }
    }

    for (section, props) in &self.sections {
      // If root section with data or generic section.
      if section != ROOT_SECTION || !props.data.is_empty() {
        if firstline {
          firstline = false;
        } else {
          // Write an empty line between sections
          writer.write_all(LineSeparator::CRLF.as_str().as_bytes())?;
        }
      }

      if section != ROOT_SECTION {
        let inherited: String = if props.inherited.is_empty() {
          String::new()
        } else {
          format!(":{}", props.inherited.join(", "))
        };

        write!(writer, "[{}]{}{}", section, inherited, LineSeparator::CRLF)?;
      }

      for (key, value) in props.iter() {
        write!(
          writer,
          "{}{}{}{}",
          key,
          DEFAULT_KV_SEPARATOR,
          value,
          LineSeparator::CRLF
        )?;
      }
    }

    writer.flush()?;

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use xrf_utils::{encode_string_to_w1251_bytes, encode_w1251_bytes_to_string};

  use crate::file::file_configuration::line_separator::DEFAULT_LINE_SEPARATOR;
  use crate::{Ltx, ROOT_SECTION};

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
    let input: String = format!("some-key = мова{}", DEFAULT_LINE_SEPARATOR);
    let ltx: Ltx = Ltx::read_from_str(&input).unwrap();

    let mut output: Vec<u8> = Vec::new();
    ltx.write_to(&mut output).unwrap();

    assert_eq!(output, encode_string_to_w1251_bytes(&input).unwrap());
    assert_eq!(encode_w1251_bytes_to_string(&output).unwrap(), input);
  }

  #[test]
  fn refuse_values_outside_windows_1251() {
    let ltx: Ltx = Ltx::read_from_str(&format!("some-key = åäö{}", DEFAULT_LINE_SEPARATOR)).unwrap();

    assert!(ltx.write_to(&mut Vec::new()).is_err());
  }
}
