use serde::{Deserialize, Serialize};

use crate::manifest::extern_callable::ExternCallable;
use crate::manifest::extern_documentation::ExternDocumentation;
use crate::manifest::extern_value::ExternValue;

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

  /// The contract as TypeScript writes it: `(name?: Type, ...) => Returns` for a callable, the type for a value.
  pub fn get_contract(&self) -> String {
    match self {
      Self::Callable(callable) => format!(
        "({}) => {}",
        callable
          .params
          .iter()
          .map(|parameter| parameter.get_contract())
          .collect::<Vec<String>>()
          .join(", "),
        callable.returns,
      ),
      Self::Value(value) => value.type_name.clone(),
    }
  }
}
