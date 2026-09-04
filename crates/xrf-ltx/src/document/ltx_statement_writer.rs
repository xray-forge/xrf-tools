use crate::syntax::LTX_LINE_SEPARATOR;

/// Formatter of LTX statements.
pub struct LtxStatementWriter {}

impl LtxStatementWriter {
  /// Write comment statement.
  pub fn write_comment(destination: &mut String, comment: &str) {
    let comment: &str = comment.trim();

    destination.push(';');

    if !comment.is_empty() {
      destination.push(' ');
      destination.push_str(comment);
    }

    destination.push_str(LTX_LINE_SEPARATOR);
  }

  /// Write include statement.
  pub fn write_include(destination: &mut String, included_path: &str, comment: Option<&str>) {
    destination.push_str(&format!("#include \"{included_path}\""));

    if let Some(comment) = comment {
      destination.push_str(&format!(" ; {}", comment));
    }

    destination.push_str(LTX_LINE_SEPARATOR);
  }

  /// Write section statement.
  /// Write a section statement carrying a DLTX operation prefix.
  ///
  /// The prefix sits outside the brackets - `![section]`, not `[!section]` - so it cannot be folded into the name.
  pub fn write_section_with_prefix(
    destination: &mut String,
    prefix: &str,
    section: &str,
    inherited: &[Box<str>],
    comment: Option<&str>,
  ) {
    if !destination.is_empty() {
      destination.push_str(LTX_LINE_SEPARATOR)
    }

    destination.push_str(&format!("{prefix}[{section}]"));

    if !inherited.is_empty() {
      destination.push_str(&format!(":{}", inherited.join(",")));
    }

    if let Some(comment) = comment {
      destination.push_str(&format!(" ; {}", comment));
    }

    destination.push_str(LTX_LINE_SEPARATOR);
  }

  /// Write section statement.
  pub fn write_key_value(destination: &mut String, key: &str, value: Option<&str>, comment: Option<&str>) {
    destination.push_str(key);

    if let Some(value) = value {
      if value.is_empty() {
        destination.push_str(" =");
      } else {
        destination.push_str(&format!(" = {value}"));
      }
    }

    if let Some(comment) = comment {
      destination.push_str(&format!(" ; {comment}"));
    }

    destination.push_str(LTX_LINE_SEPARATOR);
  }
}

#[cfg(test)]
mod test {
  use crate::document::LtxStatementWriter;

  #[test]
  fn test_write_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_comment(&mut destination, "some long comment ; nested");

    assert_eq!(destination, "; some long comment ; nested\r\n");
  }

  #[test]
  fn writes_standalone_semicolon_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_comment(&mut destination, "   ");

    assert_eq!(destination, ";\r\n");
  }

  #[test]
  fn test_write_include() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_include(&mut destination, "base\\some_file.ltx", None);

    assert_eq!(destination, "#include \"base\\some_file.ltx\"\r\n");
  }

  #[test]
  fn test_write_include_with_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_include(&mut destination, "base\\some_file.ltx", Some("nested ; comment"));

    assert_eq!(destination, "#include \"base\\some_file.ltx\" ; nested ; comment\r\n");
  }

  #[test]
  fn test_write_section() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_section_with_prefix(&mut destination, "", "some_section", &[], None);

    assert_eq!(destination, "[some_section]\r\n");
  }

  #[test]
  fn test_write_section_with_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_section_with_prefix(&mut destination, "", "some_section", &[], Some("nested ; comment"));

    assert_eq!(destination, "[some_section] ; nested ; comment\r\n");
  }

  #[test]
  fn test_write_section_inherited() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_section_with_prefix(
      &mut destination,
      "",
      "some_section",
      &[Box::from("a"), Box::from("b")],
      None,
    );

    assert_eq!(destination, "[some_section]:a,b\r\n");
  }

  #[test]
  fn test_write_section_inherited_with_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_section_with_prefix(
      &mut destination,
      "",
      "some_section",
      &[Box::from("a"), Box::from("b"), Box::from("c")],
      Some("nested ; comment"),
    );

    assert_eq!(destination, "[some_section]:a,b,c ; nested ; comment\r\n");
  }

  #[test]
  fn test_write_key() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_key_value(&mut destination, "key", None, None);

    assert_eq!(destination, "key\r\n");
  }

  #[test]
  fn test_write_key_with_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_key_value(&mut destination, "key", None, Some("test ; comment"));

    assert_eq!(destination, "key ; test ; comment\r\n");
  }

  #[test]
  fn test_write_key_with_value() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_key_value(&mut destination, "key", Some("value"), None);

    assert_eq!(destination, "key = value\r\n");
  }

  #[test]
  fn test_write_key_with_value_and_comment() {
    let mut destination: String = String::new();

    LtxStatementWriter::write_key_value(&mut destination, "key", Some("value"), Some("test ; comment"));

    assert_eq!(destination, "key = value ; test ; comment\r\n");
  }
}
