use hyper::StatusCode;

/// Every answer the transport gives other than the route's own.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TransportRefusal {
  /// No `Origin`, or one that is not the application's page.
  Forbidden,
  /// No bearer token, or not this launch's.
  Unauthorized,
  NotFound,
  MethodNotAllowed,
  /// A body that is not the route's arguments.
  BadRequest(String),
  /// The route ran and failed, with what an IPC command rejects with.
  Failed(String),
}

impl TransportRefusal {
  pub(crate) fn get_status(&self) -> StatusCode {
    match self {
      Self::Forbidden => StatusCode::FORBIDDEN,
      Self::Unauthorized => StatusCode::UNAUTHORIZED,
      Self::NotFound => StatusCode::NOT_FOUND,
      Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
      Self::BadRequest(_) => StatusCode::BAD_REQUEST,
      Self::Failed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  /// The message a caller reads, which for a failed route is exactly its error.
  pub(crate) fn get_message(&self) -> String {
    match self {
      Self::Forbidden => String::from("The transport serves the application's own page only"),
      Self::Unauthorized => String::from("The transport token is missing or is not this launch's"),
      Self::NotFound => String::from("No such transport route"),
      Self::MethodNotAllowed => String::from("Transport routes take POST"),
      Self::BadRequest(message) | Self::Failed(message) => message.clone(),
    }
  }
}
