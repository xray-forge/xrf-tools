use serde::Serialize;

/// Where the transport listens, and the token every request to it carries.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransportEndpoint {
  /// The server's origin, such as `http://127.0.0.1:52011`, which a route's path is appended to.
  pub(crate) origin: String,
  /// Sent as `Authorization: Bearer <token>`; new every launch.
  pub(crate) token: String,
}
