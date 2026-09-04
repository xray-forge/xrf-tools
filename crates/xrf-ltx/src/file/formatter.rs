use crate::LineSeparator;

/// Formatter of LTX statements.
pub struct LtxFormatter {}

impl LtxFormatter {
  /// Write comment statement.
  pub fn write_comment(destination: &mut String, comment: &str) {
    let comment: &str = comment.trim();

    destination.push(';');

    if !comment.is_empty() {
      destination.push(' ');
      destination.push_str(comment);
    }

    destination.push_str(LineSeparator::CRLF.as_str());
  }

  /// Write include statement.
  pub fn write_include(destination: &mut String, included_path: &str, comment: Option<&str>) {
    destination.push_str(&format!("#include \"{included_path}\""));

    if let Some(comment) = comment {
      destination.push_str(&format!(" ; {}", comment));
    }

    destination.push_str(LineSeparator::CRLF.as_str());
  }

  /// Write section statement.
  pub fn write_section(destination: &mut String, section: &str, inherited: Option<Vec<String>>, comment: Option<&str>) {
    Self::write_section_with_prefix(destination, "", section, inherited, comment);
  }

  /// Write a section statement carrying a DLTX operation prefix.
  ///
  /// The prefix sits outside the brackets - `![section]`, not `[!section]` - so it cannot be folded into the name.
  pub fn write_section_with_prefix(
    destination: &mut String,
    prefix: &str,
    section: &str,
    inherited: Option<Vec<String>>,
    comment: Option<&str>,
  ) {
    if !destination.is_empty() {
      destination.push_str(LineSeparator::CRLF.as_str())
    }

    destination.push_str(&format!("{prefix}[{section}]"));

    if let Some(inherited) = inherited
      && !inherited.is_empty()
    {
      destination.push_str(&format!(":{}", inherited.join(",")));
    }

    if let Some(comment) = comment {
      destination.push_str(&format!(" ; {}", comment));
    }

    destination.push_str(LineSeparator::CRLF.as_str());
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

    destination.push_str(LineSeparator::CRLF.as_str());
  }
}

#[cfg(test)]
mod test {
  use crate::file::formatter::LtxFormatter;

  #[test]
  fn test_write_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_comment(&mut destination, "some long comment ; nested");

    assert_eq!(destination, "; some long comment ; nested\r\n");
  }

  #[test]
  fn writes_standalone_semicolon_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_comment(&mut destination, "   ");

    assert_eq!(destination, ";\r\n");
  }

  #[test]
  fn test_write_include() {
    let mut destination: String = String::new();

    LtxFormatter::write_include(&mut destination, "base\\some_file.ltx", None);

    assert_eq!(destination, "#include \"base\\some_file.ltx\"\r\n");
  }

  #[test]
  fn test_write_include_with_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_include(&mut destination, "base\\some_file.ltx", Some("nested ; comment"));

    assert_eq!(destination, "#include \"base\\some_file.ltx\" ; nested ; comment\r\n");
  }

  #[test]
  fn test_write_section() {
    let mut destination: String = String::new();

    LtxFormatter::write_section(&mut destination, "some_section", None, None);

    assert_eq!(destination, "[some_section]\r\n");
  }

  #[test]
  fn test_write_section_with_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_section(
      &mut destination,
      "some_section",
      Some(Vec::new()),
      Some("nested ; comment"),
    );

    assert_eq!(destination, "[some_section] ; nested ; comment\r\n");
  }

  #[test]
  fn test_write_section_inherited() {
    let mut destination: String = String::new();

    LtxFormatter::write_section(
      &mut destination,
      "some_section",
      Some(vec![String::from("a"), String::from("b")]),
      None,
    );

    assert_eq!(destination, "[some_section]:a,b\r\n");
  }

  #[test]
  fn test_write_section_inherited_with_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_section(
      &mut destination,
      "some_section",
      Some(vec![String::from("a"), String::from("b"), String::from("c")]),
      Some("nested ; comment"),
    );

    assert_eq!(destination, "[some_section]:a,b,c ; nested ; comment\r\n");
  }

  #[test]
  fn test_write_key() {
    let mut destination: String = String::new();

    LtxFormatter::write_key_value(&mut destination, "key", None, None);

    assert_eq!(destination, "key\r\n");
  }

  #[test]
  fn test_write_key_with_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_key_value(&mut destination, "key", None, Some("test ; comment"));

    assert_eq!(destination, "key ; test ; comment\r\n");
  }

  #[test]
  fn test_write_key_with_value() {
    let mut destination: String = String::new();

    LtxFormatter::write_key_value(&mut destination, "key", Some("value"), None);

    assert_eq!(destination, "key = value\r\n");
  }

  #[test]
  fn test_write_key_with_value_and_comment() {
    let mut destination: String = String::new();

    LtxFormatter::write_key_value(&mut destination, "key", Some("value"), Some("test ; comment"));

    assert_eq!(destination, "key = value ; test ; comment\r\n");
  }
}
