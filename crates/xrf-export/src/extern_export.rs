use serde::{Deserialize, Serialize};

use crate::extern_callable::ExternCallable;
use crate::extern_documentation::ExternDocumentation;
use crate::extern_value::ExternValue;

/// One exported value. The JSON representation remains compatible with the
/// existing tracked `extern.json` artifact.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ExternExport {
  Callable(ExternCallable),
  Value(ExternValue),
}

impl ExternExport {
  pub fn source(&self) -> &str {
    match self {
      Self::Callable(value) => &value.source,
      Self::Value(value) => &value.source,
    }
  }

  pub fn documentation(&self) -> Option<&ExternDocumentation> {
    match self {
      Self::Callable(value) => value.doc.as_ref(),
      Self::Value(value) => value.doc.as_ref(),
    }
  }
}
