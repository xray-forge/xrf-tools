use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Range;

use xrf_xml::XmlElementSpan;

use crate::conversation::dialog_parse_issue_kind::DialogParseIssueKind;

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

  /// Report every attribute of an element that is not among the ones its schema defines.
  pub(crate) fn report_unknown_attributes(
    element: &XmlElementSpan,
    known: &[&str],
    dialog_id: Option<&str>,
    phrase_id: Option<&str>,
    issues: &mut Vec<Self>,
  ) {
    for (name, _) in element.attributes() {
      if !known.contains(&name) {
        issues.push(Self::new(
          DialogParseIssueKind::UnknownAttribute,
          dialog_id.map(str::to_owned),
          phrase_id.map(str::to_owned),
          name.to_owned(),
          element.element_range().clone(),
        ));
      }
    }
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
