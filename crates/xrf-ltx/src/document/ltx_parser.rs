use std::str::Chars;

use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;

use crate::document::{LtxCheck, LtxDocument, LtxItem, LtxItemKind, LtxKeyOperation, LtxSectionOperation, LtxSpan};
use crate::syntax::{
  LTX_SYMBOL_COMMENT, LTX_SYMBOL_INCLUDE, LTX_SYMBOL_INHERIT, LTX_SYMBOL_SECTION_CLOSE, LTX_SYMBOL_SECTION_OPEN,
};

/// A section header as its line spells it: the name, the parents it declares, and any trailing comment.
type ParsedSectionHeader<'a> = (&'a str, Option<Vec<&'a str>>, Option<&'a str>);

/// A field line as its text spells it: the key, its value where it has one, and any trailing comment.
type ParsedKeyValue<'a> = (&'a str, Option<&'a str>, Option<&'a str>);

/// What one statement turned out to be, with its DLTX prefix removed.
enum StatementOperation<'a> {
  Include,
  Section(LtxSectionOperation, &'a str),
  Key(LtxKeyOperation, &'a str),
}

/// Ltx parser.
pub(crate) struct LtxParser<'a> {
  char: Option<char>,
  /// The input from `char` on, so a line can be taken as a slice of it rather than rebuilt a character at a time.
  at_char: &'a str,
  reader: Chars<'a>,
  line: usize,
  column: usize,
  /// Whitespace consumed since the last line break, kept only to rebuild an authored line verbatim.
  indent: String,
  is_preserving_source: bool,
  /// Whether nothing but comments has been read yet: only that leading block may carry `@xrf-ltx` directives.
  is_metadata_header: bool,
  /// Checks the leading comment block opted out of, in the order first named.
  skipped_checks: Vec<LtxCheck>,
  /// One authored line per statement read, when the parser keeps them.
  source_lines: Vec<Box<str>>,
}

impl Default for LtxParser<'_> {
  fn default() -> Self {
    Self {
      char: None,
      at_char: "",
      indent: String::new(),
      is_preserving_source: false,
      is_metadata_header: true,
      line: 0,
      column: 0,
      reader: "".chars(),
      skipped_checks: Vec::new(),
      source_lines: Vec::new(),
    }
  }
}

impl<'a> LtxParser<'a> {
  /// The word an include statement opens with.
  const INCLUDE_DIRECTIVE: &'static str = "#include";

  /// Create new parser based on characters stream.
  pub(crate) fn new(reader: Chars<'a>) -> Self {
    let mut parser: Self = Self {
      reader,
      ..Self::default()
    };

    parser.bump();

    parser
  }

  /// A parser that also keeps each statement's line exactly as authored.
  ///
  /// For an editor that must write a file back with untouched lines byte-identical. Verification and formatting do not
  /// ask for it, so they do not pay the second copy of every line.
  pub(crate) fn new_preserving_source(reader: Chars<'a>) -> Self {
    let mut parser: Self = Self::new(reader);

    parser.is_preserving_source = true;

    parser
  }

  /// Parse the whole input into the document it was written as.
  ///
  /// The single scanning pass in this crate. Resolution, formatting, and include listing all read the result, so a
  /// command that verifies and reformats no longer scans twice.
  ///
  /// Every statement keeps its line, and each is exactly one line, so a gap between consecutive spans is a blank run.
  pub(crate) fn parse_document(&mut self) -> XrfResult<LtxDocument> {
    let mut items: Vec<LtxItem> = Vec::new();

    while let Some(item) = self.parse_next_item()? {
      items.push(LtxItem::from(item));
    }

    Ok(LtxDocument {
      items,
      skipped_checks: self.take_skipped_checks(),
      source_lines: std::mem::take(&mut self.source_lines),
    })
  }

  /// The next statement, its text borrowed from the input, or `None` once the input is spent.
  ///
  /// What [`Self::parse_document`] collects, for a reader that spends each statement as it comes rather than holding
  /// them all: a document of a few hundred thousand records otherwise holds every statement beside what it becomes.
  pub(crate) fn parse_next_item(&mut self) -> XrfResult<Option<LtxItem<&'a str>>> {
    self.skip_whitespaces();

    let Some(current_char) = self.char else {
      return Ok(None);
    };

    let span: LtxSpan = self.span_here();

    // Kept whole, comment included, because the document carries what a reformat needs to write back. The value of a
    // statement is unaffected: every `parse_*_from_line` splits the comment off itself.
    let line: &'a str = self.parse_until_eol();

    if self.is_preserving_source {
      self
        .source_lines
        .push(format!("{}{line}", self.indent).into_boxed_str());
    }

    let kind: LtxItemKind<&'a str> = if current_char == LTX_SYMBOL_COMMENT {
      // `;` is one byte, so the body starts at index 1.
      let text: &'a str = line[1..].trim();

      if self.is_metadata_header {
        // One column past the `;`, where the directive itself starts, which is what this diagnostic has always said.
        let position: LtxSpan = LtxSpan::new(span.line, span.column.saturating_add(1));

        self.parse_metadata_directive(text, position)?;
      }

      LtxItemKind::Comment { text }
    } else {
      self.is_metadata_header = false;

      match Self::split_statement_operation(line) {
        StatementOperation::Include => {
          let (path, comment) = self.parse_include_from_line(line)?;

          LtxItemKind::Include { comment, path }
        }
        StatementOperation::Section(operation, rest) => {
          let (name, parents, comment) = self.parse_section_from_line(rest)?;

          LtxItemKind::Section {
            comment,
            name,
            operation,
            parents: parents.unwrap_or_default().into_boxed_slice(),
          }
        }
        StatementOperation::Key(operation, rest) => {
          let (name, value, comment) = self.parse_key_value_from_line(rest)?;

          LtxItemKind::Key {
            comment,
            name,
            operation,
            value,
          }
        }
      }
    };

    Ok(Some(LtxItem::new(kind, span)))
  }

  /// The checks the leading comment block opted out of, taken from the parser.
  pub(crate) fn take_skipped_checks(&mut self) -> Vec<LtxCheck> {
    std::mem::take(&mut self.skipped_checks)
  }

  /// Classify one statement and strip whatever DLTX prefix it carries.
  ///
  /// No config in any reference tree begins a line with `!`, `@`, `>` or `<`, so reading those as operations cannot
  /// change how a standard file parses. Whether an operation is *allowed* is the resolver's call, not this one's.
  fn split_statement_operation(line: &str) -> StatementOperation<'_> {
    if line.starts_with(LTX_SYMBOL_INCLUDE) {
      return StatementOperation::Include;
    }

    for (prefix, operation) in [
      ("!!", LtxSectionOperation::Delete),
      ("!", LtxSectionOperation::Override),
      ("@", LtxSectionOperation::SafeOverride),
    ] {
      if let Some(rest) = line.strip_prefix(prefix)
        && rest.starts_with(LTX_SYMBOL_SECTION_OPEN)
      {
        return StatementOperation::Section(operation, rest);
      }
    }

    if line.starts_with(LTX_SYMBOL_SECTION_OPEN) {
      return StatementOperation::Section(LtxSectionOperation::Declare, line);
    }

    for (prefix, operation) in [
      ("!", LtxKeyOperation::Delete),
      (">", LtxKeyOperation::ListAppend),
      ("<", LtxKeyOperation::ListRemove),
    ] {
      if let Some(rest) = line.strip_prefix(prefix) {
        return StatementOperation::Key(operation, rest);
      }
    }

    StatementOperation::Key(LtxKeyOperation::Set, line)
  }

  fn parse_metadata_directive(&mut self, comment: &str, span: LtxSpan) -> XrfResult {
    let (line, column) = (span.get_line(), span.get_column());

    let mut parts: std::str::SplitWhitespace<'_> = comment.split_whitespace();

    if parts.next() != Some("@xrf-ltx") {
      return Ok(());
    }

    let Some(directive) = parts.next() else {
      return Err(XrfError::new_ltx_parse_error(
        line,
        column,
        "Expected an @xrf-ltx directive name",
      ));
    };

    if parts.next().is_some() {
      return Err(XrfError::new_ltx_parse_error(
        line,
        column,
        "Expected exactly one @xrf-ltx directive",
      ));
    }

    let Some(check) = LtxCheck::from_skip_directive(directive) else {
      return Err(XrfError::new_ltx_parse_error(
        line,
        column,
        format!("Unknown @xrf-ltx directive '{directive}'"),
      ));
    };

    if !self.skipped_checks.contains(&check) {
      self.skipped_checks.push(check);
    }

    Ok(())
  }
}

impl<'a> LtxParser<'a> {
  /// Where the cursor is, as a statement records it.
  ///
  /// `column` already counts from one and names the character now under the cursor, which is this statement's first.
  /// `line` counts from zero. `error` reports `column + 1` instead, meaning the position after what it consumed, and
  /// the metadata diagnostic keeps that older reading.
  fn span_here(&self) -> LtxSpan {
    LtxSpan::new(
      u32::try_from(self.line + 1).unwrap_or(u32::MAX),
      u32::try_from(self.column).unwrap_or(u32::MAX),
    )
  }

  fn bump(&mut self) {
    self.at_char = self.reader.as_str();
    self.char = self.reader.next();

    match self.char {
      Some('\n') => {
        self.line += 1;
        self.column = 0;
      }
      Some(..) => {
        self.column += 1;
      }
      None => {}
    }
  }

  /// Create parsing error.
  fn error<U, M: Into<String>>(&self, message: M) -> XrfResult<U> {
    Err(XrfError::new_ltx_parse_error(self.line + 1, self.column + 1, message))
  }

  /// Consume all the white space until the end of the line or a tab.
  fn skip_whitespaces(&mut self) {
    self.indent.clear();

    while let Some(char) = self.char {
      if !char.is_whitespace() && char != '\n' && char != '\t' && char != '\r' {
        break;
      }

      // The run after the last break is the indentation of the statement about to be read, which is the one part of an
      // authored line the statement text itself cannot carry.
      if self.is_preserving_source {
        if char == '\n' || char == '\r' {
          self.indent.clear();
        } else {
          self.indent.push(char);
        }
      }

      self.bump();
    }
  }

  /// The rest of the current line, leaving the cursor on the break that ends it.
  ///
  /// Taken as one slice of the input rather than a character at a time: no break lies inside the line, so the cursor
  /// only moves along it, one column per character, exactly as stepping through it would.
  fn parse_until_eol(&mut self) -> &'a str {
    if matches!(self.char, None | Some('\n' | '\r')) {
      return "";
    }

    let text: &'a str = self.at_char;
    // Searched as bytes: both breaks are ASCII, which no byte of a wider character can be.
    let end: usize = text
      .bytes()
      .position(|byte| byte == b'\n' || byte == b'\r')
      .unwrap_or(text.len());
    let line: &'a str = &text[..end];

    // The line's first character is the one under the cursor, already counted.
    self.column += line.chars().count() - 1;
    self.reader = text[line.len()..].chars();
    self.bump();

    line
  }
}

impl LtxParser<'_> {
  /// Parse section name, inherited sections and comment from the line.
  fn parse_section_from_line<'line>(&self, line: &'line str) -> XrfResult<ParsedSectionHeader<'line>> {
    if line.is_empty() {
      return self.error("Failed to parse empty section statement");
    }

    let Some(section_ends_at) = line.find(LTX_SYMBOL_SECTION_CLOSE) else {
      return self.error("Failed to parse section statement without closing bracket ']'");
    };

    let section: &str = &line[1..section_ends_at];
    let remainder: &str = line[section_ends_at + 1..].trim();

    if remainder.is_empty() {
      Ok((section, None, None))
    } else if let Some(remainder) = remainder.strip_prefix(LTX_SYMBOL_INHERIT) {
      let (inherited, comment) = match remainder.find(LTX_SYMBOL_COMMENT) {
        Some(position) => (&remainder[0..position], Some(&remainder[position + 1..])),
        None => (remainder, None),
      };

      let inherited: Vec<&str> = inherited
        .split(',')
        .map(str::trim)
        .filter(|it| !it.is_empty())
        .collect::<Vec<&str>>();

      Ok((
        section,
        if inherited.is_empty() { None } else { Some(inherited) },
        comment.map(str::trim),
      ))
    } else {
      // Anything else after the brackets is no part of the header, and the engine reads past it: shipped configs write
      // column captions there (`[m_dog_step_params]   fwdleftleg ...`). Kept whole as the header's comment, so a
      // reformat writes it back behind a `;` rather than dropping any of it.
      let comment: &str = remainder
        .strip_prefix(LTX_SYMBOL_COMMENT)
        .unwrap_or(remainder)
        .trim_start();

      Ok((section, None, (!comment.is_empty()).then_some(comment)))
    }
  }

  /// Parse the file an include statement names, and its comment.
  ///
  /// The name is what sits between the first two quotes after the directive, which is how the engine's `CInifile` reads
  /// it, so `#include "file.ltx"`, `#include("file.ltx")` and any spacing around the quotes all name `file.ltx`.
  fn parse_include_from_line<'line>(&self, line: &'line str) -> XrfResult<(&'line str, Option<&'line str>)> {
    let (include, comment) = match line.trim().split_once(LTX_SYMBOL_COMMENT) {
      Some((include, comment)) => (include.trim(), Some(comment.trim())),
      None => (line.trim(), None),
    };

    let Some(name) = include
      .strip_prefix(Self::INCLUDE_DIRECTIVE)
      .and_then(|rest| rest.split('"').nth(1))
    else {
      return self.error(format!(
        "Expected correct '#include \"config.ltx\"' statement, got '{include}'"
      ));
    };

    if name.is_empty() {
      return self.error(String::from(
        "Expected valid file name in include statement, got empty file name",
      ));
    }

    if !XrayExtension::Ltx.matches(name) {
      return self.error(format!(
        "Included file should have .{} extension, got '{name}'",
        XrayExtension::Ltx
      ));
    }

    Ok((name, comment.filter(|it| !it.is_empty())))
  }

  /// Parse line key, value and comment from provided line.
  fn parse_key_value_from_line<'line>(&self, line: &'line str) -> XrfResult<ParsedKeyValue<'line>> {
    if line.is_empty() {
      return self.error("Failed to parse empty value statement");
    }

    let (data, comment) = match line.split_once(LTX_SYMBOL_COMMENT) {
      None => (line.trim(), None),
      Some((data, comment)) => (data.trim(), Some(comment.trim())),
    };

    let (key, value) = match data.split_once('=') {
      None => (data.trim(), None),
      Some((key, value)) => (key.trim(), Some(value.trim())),
    };

    Ok((key, value, comment.filter(|it| !it.is_empty())))
  }
}

#[cfg(test)]
mod test {
  use crate::document::{LtxCheck, LtxParser};
  use crate::ltx::Ltx;

  #[test]
  fn parses_header_metadata_directive() {
    let ltx: Ltx = Ltx::read_from_str("; @xrf-ltx skip-inheritance\n[child]:missing\n").unwrap();

    assert!(ltx.is_check_skipped(LtxCheck::Inheritance));
    assert!(ltx.into_inherited().is_ok());
  }

  #[test]
  fn rejects_unknown_header_metadata_directive() {
    let error = Ltx::read_from_str("; @xrf-ltx skip-unknown\n[section]\n").unwrap_err();

    assert_eq!(
      error.to_string(),
      "Ltx parse error: 1:2 Unknown @xrf-ltx directive 'skip-unknown'"
    );
  }

  /// A line is taken whole, but the cursor still counts one column per character, wide or not, and a lone `\r` ends a
  /// line without starting a new one, as stepping through the text a character at a time did.
  #[test]
  fn places_an_error_by_characters_after_wide_text_and_lone_returns() {
    for (input, expected_line, expected_column) in [
      ("ключ = 1\n[не", 2, 4),
      ("a = 1\r[не\r", 1, 11),
      // Reported where the cursor stands, past the break that ended the line.
      ("a = 1\r\n\r\n  [не ; ж\nb = 2", 4, 1),
      ("a = 1\r\n\r\n  [не ; ж\rb = 2", 3, 11),
    ] {
      match Ltx::read_from_str(input).expect_err("an unclosed header") {
        xrf_error::XrfError::LtxParse { line, col, .. } => {
          assert_eq!((line, col), (expected_line, expected_column), "{input:?}");
        }
        error => panic!("Unexpected error {error}"),
      }
    }
  }

  #[test]
  fn ignores_metadata_directive_after_ltx_content() {
    let ltx: Ltx = Ltx::read_from_str("[section]\n; @xrf-ltx skip-inheritance\n").unwrap();

    assert!(!ltx.is_check_skipped(LtxCheck::Inheritance));
  }

  #[test]
  fn test_read_section() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section]").unwrap(),
      ("section", None, None)
    );
  }

  #[test]
  fn test_read_section_with_inherited() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section] : a,   b, c").unwrap(),
      ("section", Some(vec!("a", "b", "c",)), None)
    );
  }

  #[test]
  fn test_read_section_with_empty_inherited() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section] :  ").unwrap(),
      ("section", None, None)
    );
  }

  #[test]
  fn test_read_section_with_empty_inherited_comment() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section] :  ;;;; test").unwrap(),
      ("section", None, Some(";;; test"))
    );
  }

  #[test]
  fn test_read_section_with_inherited_comment() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser
        .parse_section_from_line("[section] : a,  b    ;   commented phrase ")
        .unwrap(),
      ("section", Some(vec!("a", "b")), Some("commented phrase"))
    );
  }

  #[test]
  fn test_read_section_with_comment() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section];commented phrase ").unwrap(),
      ("section", None, Some("commented phrase"))
    );
  }

  /// Shipped configs write column captions after a header, and a reformat has to keep every character of them:
  /// `[m_dog_step_params]   fwdleftleg` becomes a comment reading `fwdleftleg`, not `wdleftleg`.
  #[test]
  fn keeps_text_after_a_header_whole_as_its_comment() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser
        .parse_section_from_line("[m_dog_step_params]   fwdleftleg   fwdrightleg")
        .unwrap(),
      ("m_dog_step_params", None, Some("fwdleftleg   fwdrightleg"))
    );
    assert_eq!(
      parser.parse_section_from_line("[logic]r").unwrap(),
      ("logic", None, Some("r"))
    );
  }

  /// A header followed by a character wider than a byte is read, never sliced inside that character.
  #[test]
  fn reads_a_header_followed_by_non_ascii_text() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[marsh_20]Жовтий").unwrap(),
      ("marsh_20", None, Some("Жовтий"))
    );
  }

  #[test]
  fn names_the_file_between_the_first_two_quotes_of_an_include() {
    let parser: LtxParser<'_> = Default::default();

    for line in [
      "#include \"file.ltx\"",
      "#include(\"file.ltx\")",
      "#include   \"file.ltx\"",
      "#include\t\"file.ltx\" trailing",
    ] {
      assert_eq!(
        parser.parse_include_from_line(line).unwrap(),
        ("file.ltx", None),
        "for {line}"
      );
    }

    assert_eq!(
      parser.parse_include_from_line("#include \"file.ltx\" ; why").unwrap(),
      ("file.ltx", Some("why"))
    );
  }

  /// A malformed include is refused with an error, whatever characters follow the directive.
  #[test]
  fn refuses_an_unquoted_include_rather_than_aborting() {
    let parser: LtxParser<'_> = Default::default();

    assert!(parser.parse_include_from_line("#include ЖЖ.ltx").is_err());
    assert!(Ltx::read_from_str("#include ЖЖЖЖЖ.ltx\n").is_err());
  }

  #[test]
  fn test_read_key_value() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_key_value_from_line("  key   =   value").unwrap(),
      ("key", Some("value"), None)
    );
  }

  #[test]
  fn test_read_key_value_comment() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser
        .parse_key_value_from_line("  key   =   1     ;   some phrase")
        .unwrap(),
      ("key", Some("1"), Some("some phrase"))
    );
  }

  #[test]
  fn test_read_key_only() {
    let parser: LtxParser<'_> = Default::default();

    assert_eq!(
      parser.parse_key_value_from_line("  key   ").unwrap(),
      ("key", None, None)
    );
  }
}
