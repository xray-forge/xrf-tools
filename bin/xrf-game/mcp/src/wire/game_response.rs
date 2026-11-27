use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The game's answer to one request, a line on its standard output with the request's id: what it found, or why it
/// could not.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameResponse {
  pub id: String,
  pub ok: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub result: Option<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

impl GameResponse {
  pub fn succeed(id: &str, result: Value) -> Self {
    Self {
      id: id.to_owned(),
      ok: true,
      result: Some(result),
      error: None,
    }
  }

  pub fn fail(id: &str, error: impl Into<String>) -> Self {
    Self {
      id: id.to_owned(),
      ok: false,
      result: None,
      error: Some(error.into()),
    }
  }
}
