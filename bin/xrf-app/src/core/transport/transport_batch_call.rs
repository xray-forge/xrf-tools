use serde::Deserialize;

/// One call of a batch: a route, and the arguments its own request would carry as its body.
#[derive(Debug, Deserialize)]
pub(crate) struct TransportBatchCall {
  pub(crate) route: String,
  pub(crate) args: serde_json::Value,
}
