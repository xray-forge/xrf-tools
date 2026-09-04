use std::str::Chars;

use xrf_error::{XrfError, XrfResult};

use crate::LtxCheck;
use crate::document::ltx_document::LtxDocument;
use crate::document::ltx_item::{LtxItem, LtxItemKind};
use crate::document::ltx_key_operation::LtxKeyOperation;
use crate::document::ltx_section_operation::LtxSectionOperation;
use crate::document::ltx_span::LtxSpan;
use crate::file::file_configuration::constants::{
  LTX_SYMBOL_COMMENT, LTX_SYMBOL_INCLUDE, LTX_SYMBOL_INHERIT, LTX_SYMBOL_SECTION_CLOSE, LTX_SYMBOL_SECTION_OPEN,
};

/// What one statement turned out to be, with its DLTX prefix removed.
enum StatementOperation<'a> {
  Include,
  Section(LtxSectionOperation, &'a str),
  Key(LtxKeyOperation, &'a str),
}

/// Ltx parser.
pub struct LtxParser<'a> {
  char: Option<char>,
  reader: Chars<'a>,
  line: usize,
  column: usize,
  /// Whitespace consumed since the last line break, kept only to rebuild an authored line verbatim.
  indent: String,
  is_preserving_source: bool,
}

impl Default for LtxParser<'_> {
  fn default() -> Self {
    Self {
      char: None,
      indent: String::new(),
      is_preserving_source: false,
      line: 0,
      column: 0,
      reader: "".chars(),
    }
  }
}

impl<'a> LtxParser<'a> {
  /// Create new parser based on characters stream.
  pub fn new(reader: Chars<'a>) -> Self {
    let mut parser: Self = Self {
      char: None,
      indent: String::new(),
      is_preserving_source: false,
      line: 0,
      column: 0,
      reader,
    };

    parser.bump();

    parser
  }

  /// A parser that also keeps each statement's line exactly as authored.
  ///
  /// For an editor that must write a file back with untouched lines byte-identical. Verification and formatting do not
  /// ask for it, so they do not pay the second copy of every line.
  pub fn new_preserving_source(reader: Chars<'a>) -> Self {
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
  pub fn parse_document(&mut self) -> XrfResult<LtxDocument> {
    let mut document: LtxDocument = LtxDocument::default();
    let mut is_metadata_header: bool = true;

    self.skip_whitespaces();

    while let Some(current_char) = self.char {
      // `column` already counts from one and names the character now under the cursor, which is this statement's first.
      // `line` counts from zero. `error` reports `column + 1` instead, meaning the position after what it consumed, and
      // the metadata diagnostic below keeps that older reading.
      let span: LtxSpan = LtxSpan::new(self.line + 1, self.column);

      // Kept whole, comment included, because the document carries what a reformat needs to write back. The value of a
      // statement is unaffected: every `parse_*_from_line` splits the comment off itself.
      let line: String = self.parse_until_eol(false)?;
      let source: Option<String> = self.is_preserving_source.then(|| format!("{}{line}", self.indent));

      let kind: LtxItemKind = if current_char == LTX_SYMBOL_COMMENT {
        // `;` is one byte, so the body starts at index 1.
        let text: String = String::from(line[1..].trim());

        if is_metadata_header {
          // One column past the `;`, where the directive itself starts, which is what this diagnostic has always said.
          let position: LtxSpan = LtxSpan::new(span.line, span.column + 1);

          self.parse_metadata_directive(&text, &mut document, position)?;
        }

        LtxItemKind::Comment { text }
      } else {
        is_metadata_header = false;

        match Self::split_statement_operation(&line) {
          StatementOperation::Include => {
            let (path, comment) = self.parse_include_from_line(&line)?;

            LtxItemKind::Include { comment, path }
          }
          StatementOperation::Section(operation, rest) => {
            let (name, parents, comment) = self.parse_section_from_line(rest)?;

            LtxItemKind::Section {
              comment,
              name,
              operation,
              parents: parents.unwrap_or_default(),
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

      document.items.push(LtxItem::new(kind, span).with_source(source));

      self.skip_whitespaces();
    }

    Ok(document)
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

  fn parse_metadata_directive(&self, comment: &str, document: &mut LtxDocument, span: LtxSpan) -> XrfResult {
    let (line, column) = (span.line, span.column);

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

    document.record_skipped_check(check);

    Ok(())
  }
}

impl LtxParser<'_> {
  fn bump(&mut self) {
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

  /// Consume all the white space except line break.
  fn skip_whitespaces_except_line_break(&mut self) {
    while let Some(c) = self.char {
      if (c == '\n' || c == '\r' || !c.is_whitespace()) && c != '\t' {
        break;
      }

      self.bump();
    }
  }

  fn skip_comment(&mut self) -> XrfResult<String> {
    self.bump();

    // Allow empty value.
    self.skip_whitespaces_except_line_break();

    match self.char {
      None => Ok(String::new()),
      _ => self.parse_until_eol(false),
    }
  }

  fn parse_until(&mut self, endpoint: &[Option<char>], check_inline_comment: bool) -> XrfResult<String> {
    let mut result: String = String::new();

    while !endpoint.contains(&self.char) {
      match self.char {
        None => {
          return self.error(format!("Expecting \"{:?}\" but found EOF.", endpoint));
        }
        Some(space) if check_inline_comment && (space == ' ' || space == '\t') => {
          self.bump();

          match self.char {
            Some(';') => {
              // [space]; starts an inline comment.
              break;
            }
            Some(_) => {
              result.push(space);
              continue;
            }
            None => {
              result.push(space);
            }
          }
        }
        Some(c) => {
          result.push(c);
        }
      }
      self.bump();
    }

    let _ = check_inline_comment;
    Ok(result)
  }

  #[inline]
  fn parse_until_eol(&mut self, strip_inline_comment: bool) -> XrfResult<String> {
    let value: String = self.parse_until(&[Some('\n'), Some('\r'), None], strip_inline_comment)?;

    if strip_inline_comment && matches!(self.char, Some(LTX_SYMBOL_COMMENT)) {
      self.skip_comment()?;
    }

    Ok(value)
  }
}

impl LtxParser<'_> {
  /// Parse section name, inherited sections and comment from the line.
  fn parse_section_from_line(&self, line: &str) -> XrfResult<(String, Option<Vec<String>>, Option<String>)> {
    if line.is_empty() {
      return self.error("Failed to parse empty section statement");
    }

    let closing_bracket_position: Option<usize> = line.find(LTX_SYMBOL_SECTION_CLOSE);

    if closing_bracket_position.is_none() {
      return self.error("Failed to parse section statement without closing bracket ']'");
    }

    let section_ends_at: usize = closing_bracket_position.unwrap();
    let section: String = String::from(&line[1..section_ends_at]);
    let remainder: &str = line[section_ends_at + 1..].trim();

    if remainder.is_empty() {
      Ok((section, None, None))
    } else if let Some(remainder) = remainder.strip_prefix(LTX_SYMBOL_INHERIT) {
      let (inherited, comment) = match remainder.find(LTX_SYMBOL_COMMENT) {
        Some(position) => (&remainder[0..position], Some(&remainder[position + 1..])),
        None => (remainder, None),
      };

      let inherited: Vec<String> = inherited
        .split(',')
        .filter_map(|it| {
          let it: &str = it.trim();

          if it.is_empty() { None } else { Some(String::from(it)) }
        })
        .collect::<Vec<String>>();

      Ok((
        section,
        if inherited.is_empty() { None } else { Some(inherited) },
        comment.map(|comment| String::from(comment.trim())),
      ))
    } else {
      // Fully trimmed value after splitting.
      let comment: String = String::from(remainder[1..].trim_start());

      Ok((section, None, if comment.is_empty() { None } else { Some(comment) }))
    }
  }

  /// Parse section name, inherited sections and comment from the line.
  ///
  /// Supported include variants are:
  /// - #include "file.ltx"
  /// - #include("file.ltx")
  fn parse_include_from_line(&self, line: &str) -> XrfResult<(String, Option<String>)> {
    if line.is_empty() {
      return self.error("Failed to parse empty include statement");
    }

    let line: &str = line.trim();

    let (include, comment) = match line.split_once(';') {
      Some((key, value)) => (key.trim(), Some(value.trim())),
      None => (line, None),
    };

    let included_path: String = if include.starts_with("#include \"") && include.ends_with('\"') {
      String::from(&include[10..include.len() - 1])
    } else if include.starts_with("#include(\"") && include.ends_with("\")") {
      String::from(&include[10..include.len() - 2])
    } else if include.len() > 10 {
      if let Some(closing_index) = include[10..].find("\"") {
        // Closing index is -10 positions:
        String::from(&include[10..closing_index + 10])
      } else {
        return self.error(format!(
          "Expected correct '#include \"config.ltx\"' statement, got '{include}'"
        ));
      }
    } else {
      return self.error(format!(
        "Expected correct '#include \"config.ltx\"' statement, got '{include}'"
      ));
    };

    if included_path.is_empty() {
      return self.error(String::from(
        "Expected valid file name in include statement, got empty file name",
      ));
    }

    if !included_path.ends_with(".ltx") {
      return self.error(format!(
        "Included file should have .ltx extension, got '{included_path}'",
      ));
    }

    Ok((included_path, comment.filter(|it| !it.is_empty()).map(String::from)))
  }

  /// Parse line key, value and comment from provided line.
  fn parse_key_value_from_line(&self, line: &str) -> XrfResult<(String, Option<String>, Option<String>)> {
    if line.is_empty() {
      return self.error("Failed to parse empty value statement");
    }

    let (data, comment) = match line.split_once(';') {
      None => (line.trim(), None),
      Some((data, comment)) => (data.trim(), Some(comment.trim())),
    };

    let (key, value) = match data.split_once('=') {
      None => (data.trim(), None),
      Some((key, value)) => (key.trim(), Some(value.trim())),
    };

    Ok((
      String::from(key),
      value.map(String::from),
      comment.filter(|it| !it.is_empty()).map(String::from),
    ))
  }
}

#[cfg(test)]
mod test {
  use crate::file::parser::LtxParser;
  use crate::{Ltx, LtxCheck};

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

  #[test]
  fn ignores_metadata_directive_after_ltx_content() {
    let ltx: Ltx = Ltx::read_from_str("[section]\n; @xrf-ltx skip-inheritance\n").unwrap();

    assert!(!ltx.is_check_skipped(LtxCheck::Inheritance));
  }

  #[test]
  fn test_read_section() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section]").unwrap(),
      (String::from("section"), None, None)
    );
  }

  #[test]
  fn test_read_section_with_inherited() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section] : a,   b, c").unwrap(),
      (
        String::from("section"),
        Some(vec!(String::from("a"), String::from("b"), String::from("c"),)),
        None
      )
    );
  }

  #[test]
  fn test_read_section_with_empty_inherited() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section] :  ").unwrap(),
      (String::from("section"), None, None)
    );
  }

  #[test]
  fn test_read_section_with_empty_inherited_comment() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section] :  ;;;; test").unwrap(),
      (String::from("section"), None, Some(String::from(";;; test")))
    );
  }

  #[test]
  fn test_read_section_with_inherited_comment() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser
        .parse_section_from_line("[section] : a,  b    ;   commented phrase ")
        .unwrap(),
      (
        String::from("section"),
        Some(vec!(String::from("a"), String::from("b"))),
        Some(String::from("commented phrase"))
      )
    );
  }

  #[test]
  fn test_read_section_with_comment() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_section_from_line("[section];commented phrase ").unwrap(),
      (String::from("section"), None, Some(String::from("commented phrase")))
    );
  }

  #[test]
  fn test_read_key_value() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_key_value_from_line("  key   =   value").unwrap(),
      (String::from("key"), Some(String::from("value")), None)
    );
  }

  #[test]
  fn test_read_key_value_comment() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser
        .parse_key_value_from_line("  key   =   1     ;   some phrase")
        .unwrap(),
      (
        String::from("key"),
        Some(String::from("1")),
        Some(String::from("some phrase"))
      )
    );
  }

  #[test]
  fn test_read_key_only() {
    let parser: LtxParser = Default::default();

    assert_eq!(
      parser.parse_key_value_from_line("  key   ").unwrap(),
      (String::from("key"), None, None)
    );
  }
}
