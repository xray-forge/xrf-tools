/// Escaping for values spliced into XML written by hand.
///
/// Only what XML requires is escaped. Escaping more is valid but not harmless: shipped game data writes apostrophes,
/// quotes and `>` raw everywhere, so rewriting them as entities changes bytes an edit never meant to touch and diverges
/// from the file's own convention.
pub struct XmlEscape;

impl XmlEscape {
  /// Escape a value for use as XML character data.
  ///
  /// Escapes `&` and `<`, and the `>` of a `]]>`, the one sequence character data may not contain. Every other `>`
  /// stays raw. Pass a raw value: an existing entity is escaped again.
  pub fn text(value: &str) -> String {
    value
      .replace('&', "&amp;")
      .replace('<', "&lt;")
      .replace("]]>", "]]&gt;")
  }

  /// Escape a value for use inside a double-quoted attribute.
  ///
  /// Adds the quote character to what character data needs, and writes tabs and line breaks as character references,
  /// since a parser reads a literal one in an attribute back as a space. The apostrophe is left raw because these
  /// writers quote attributes with `"`, so `'` carries no meaning inside one.
  pub fn attribute(value: &str) -> String {
    Self::text(value)
      .replace('"', "&quot;")
      .replace('\t', "&#9;")
      .replace('\n', "&#10;")
      .replace('\r', "&#13;")
  }
}

#[cfg(test)]
mod tests {
  use crate::document::XmlDocument;
  use crate::write::XmlEscape;
  use crate::xml_parse_options::XmlParseOptions;

  #[test]
  fn text_escapes_only_what_character_data_requires() {
    assert_eq!(XmlEscape::text("a & b < c"), "a &amp; b &lt; c");
  }

  #[test]
  fn text_leaves_alone_what_shipped_data_writes_raw() {
    // A dialogue line like "we'll jump that jackass" must come back out byte for byte.
    assert_eq!(XmlEscape::text("we'll go > there \"now\""), "we'll go > there \"now\"");
  }

  #[test]
  fn text_escapes_the_end_of_a_cdata_section() {
    let escaped: String = XmlEscape::text("a ]]> b ]] > c");

    assert_eq!(escaped, "a ]]&gt; b ]] > c");
    assert_eq!(
      XmlDocument::parse(&format!("<root>{escaped}</root>"), XmlParseOptions::default())
        .unwrap()
        .root()
        .text(),
      "a ]]> b ]] > c"
    );
  }

  #[test]
  fn attributes_also_escape_the_quote_they_are_wrapped_in() {
    assert_eq!(XmlEscape::attribute("a \" b & c"), "a &quot; b &amp; c");
  }

  #[test]
  fn attributes_leave_the_apostrophe_raw() {
    // Valid inside a double-quoted attribute, and one fewer entity in the output.
    assert_eq!(XmlEscape::attribute("it's"), "it's");
  }

  #[test]
  fn attributes_keep_their_whitespace_through_a_parse() {
    let value: &str = "a\tb\nc\r\nd";
    let document: XmlDocument = XmlDocument::parse(
      &format!("<root value=\"{}\"/>", XmlEscape::attribute(value)),
      XmlParseOptions::default(),
    )
    .unwrap();

    assert_eq!(document.root().attribute("value"), Some(value));
  }

  #[test]
  fn escaping_an_already_escaped_value_escapes_its_ampersand_again() {
    assert_eq!(XmlEscape::text("&amp;"), "&amp;amp;");
  }
}
