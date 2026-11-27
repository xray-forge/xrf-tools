use std::slice::Iter;

use crate::issue::config_issue::ConfigIssue;

/// The issues a read of the configs collected, in the order found. A read never stops at one: what does not read takes
/// its default or is left out, and the issue says which, so one mistyped value never stops a game, as one broken line
/// never stops X-Ray's script.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigIssues {
  issues: Vec<ConfigIssue>,
}

impl ConfigIssues {
  pub fn push(&mut self, issue: ConfigIssue) {
    self.issues.push(issue);
  }

  /// An issue of the whole `file`, such as one the tree lacks.
  pub fn push_file(&mut self, file: &str, message: impl Into<String>) {
    self.push(ConfigIssue {
      file: String::from(file),
      section: None,
      key: None,
      message: message.into(),
    });
  }

  pub fn len(&self) -> usize {
    self.issues.len()
  }

  pub fn is_empty(&self) -> bool {
    self.issues.is_empty()
  }

  pub fn iter(&self) -> Iter<'_, ConfigIssue> {
    self.issues.iter()
  }
}

impl<'a> IntoIterator for &'a ConfigIssues {
  type Item = &'a ConfigIssue;
  type IntoIter = Iter<'a, ConfigIssue>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}
