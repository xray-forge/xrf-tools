use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::core::transport::{TransportAnswer, TransportRefusal};
use crate::core::types::TauriResult;

/// What calling a route comes to.
pub(crate) type TransportFuture = Pin<Box<dyn Future<Output = Result<TransportAnswer, TransportRefusal>> + Send>>;

/// A route's handler with its arguments still a JSON body.
type TransportHandler<C> = Arc<dyn Fn(C, Bytes) -> TransportFuture + Send + Sync>;

/// One route: `<plugin>/<name>`, answering bytes for the arguments its body carries.
pub(crate) struct TransportRoute<C> {
  path: String,
  handler: TransportHandler<C>,
}

impl<C: Send + 'static> TransportRoute<C> {
  /// A route over a typed handler, which reads its arguments as a command reads its own.
  pub(crate) fn new<Q, F, A>(plugin: &str, name: &str, handler: F) -> Self
  where
    Q: DeserializeOwned + Send + 'static,
    F: Fn(C, Q) -> A + Send + Sync + 'static,
    A: Future<Output = TauriResult<TransportAnswer>> + Send + 'static,
  {
    let path: String = format!("{plugin}/{name}");
    let described: String = path.clone();
    let handler: Arc<F> = Arc::new(handler);

    Self {
      path,
      handler: Arc::new(move |context: C, body: Bytes| -> TransportFuture {
        let handler: Arc<F> = Arc::clone(&handler);
        let described: String = described.clone();

        Box::pin(async move {
          let request: Q = serde_json::from_slice(&body).map_err(|error| {
            TransportRefusal::BadRequest(format!("Arguments of '{described}' do not read: {error}"))
          })?;

          handler(context, request).await.map_err(TransportRefusal::Failed)
        })
      }),
    }
  }

  pub(crate) fn get_path(&self) -> &str {
    &self.path
  }

  pub(crate) fn call(&self, context: C, body: Bytes) -> TransportFuture {
    (self.handler)(context, body)
  }
}
