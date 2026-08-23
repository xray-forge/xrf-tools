use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Range;

/// Something the parser could read but the schema does not describe.
///
/// Reading never fails on these: shipped dialog data holds all of them, and a reader that refused
/// would be unable to open the files it exists to inspect. Validation turns them into findings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogParseIssueKind {
  /// An element the schema does not define at this level.
  UnknownElement,
  /// An attribute the schema does not define at this level.
  UnknownAttribute,
  /// A `dialog` or `phrase` with no `id`, which cannot be addressed and is skipped.
  MissingId,
  /// A `priority` attribute that is not an integer. The dialog is kept without one.
  InvalidPriority,
}

/// One issue, with where it was found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogParseIssue {
  kind: DialogParseIssueKind,
  /// Dialog the issue sits in, or `None` when the dialog itself has no id.
  dialog_id: Option<String>,
  /// Phrase the issue sits in, when it is below one.
  phrase_id: Option<String>,
  /// What was found: an element name, an attribute name, or an unparsable value.
  subject: String,
  range: Range<usize>,
}

impl DialogParseIssue {
  pub fn new(
    kind: DialogParseIssueKind,
    dialog_id: Option<String>,
    phrase_id: Option<String>,
    subject: String,
    range: Range<usize>,
  ) -> Self {
    Self {
      kind,
      dialog_id,
      phrase_id,
      subject,
      range,
    }
  }

  pub fn get_kind(&self) -> DialogParseIssueKind {
    self.kind
  }

  pub fn get_dialog_id(&self) -> Option<&str> {
    self.dialog_id.as_deref()
  }

  pub fn get_phrase_id(&self) -> Option<&str> {
    self.phrase_id.as_deref()
  }

  pub fn get_subject(&self) -> &str {
    &self.subject
  }

  pub fn get_range(&self) -> &Range<usize> {
    &self.range
  }
}

impl Display for DialogParseIssue {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    let location: String = match (&self.dialog_id, &self.phrase_id) {
      (Some(dialog), Some(phrase)) => format!("{dialog}#{phrase}"),
      (Some(dialog), None) => dialog.clone(),
      _ => String::from("<unnamed>"),
    };

    match self.kind {
      DialogParseIssueKind::UnknownElement => write!(formatter, "{location}: unknown element '{}'", self.subject),
      DialogParseIssueKind::UnknownAttribute => write!(formatter, "{location}: unknown attribute '{}'", self.subject),
      DialogParseIssueKind::MissingId => write!(formatter, "{location}: {} without an id", self.subject),
      DialogParseIssueKind::InvalidPriority => {
        write!(formatter, "{location}: priority '{}' is not an integer", self.subject)
      }
    }
  }
}
