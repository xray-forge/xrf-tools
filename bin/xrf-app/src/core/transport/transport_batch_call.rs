use serde::Deserialize;
use serde_json::value::RawValue;

/// One call of a batch: a route, and the arguments its own request would carry as its body, as they were sent.
#[derive(Debug, Deserialize)]
pub(crate) struct TransportBatchCall {
  pub(crate) route: String,
  pub(crate) args: Box<RawValue>,
}
