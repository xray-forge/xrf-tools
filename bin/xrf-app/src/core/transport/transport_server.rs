use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener as StdTcpListener};
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Incoming;
use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::core::transport::{
  TransportAnswer, TransportBatchCall, TransportBody, TransportOrigins, TransportPart, TransportRefusal,
  TransportRoute, TransportRoutes, TransportToken,
};

/// The loopback HTTP/1.1 server the routes are served from, one connection kept alive per fetcher.
pub(crate) struct TransportServer<C> {
  routes: TransportRoutes<C>,
  origins: TransportOrigins,
  token: TransportToken,
  context: C,
}

impl<C: Clone + Send + Sync + 'static> TransportServer<C> {
  /// Largest request body read, far above any route's arguments.
  const MAXIMUM_BODY_SIZE: usize = 64 * 1024;

  /// Where a batch is posted: a JSON array of calls, each a route and its arguments, answered by their parts.
  pub(crate) const BATCH_PATH: &'static str = "batch";

  /// Largest batch body read, its calls' arguments together.
  const MAXIMUM_BATCH_BODY_SIZE: usize = 1024 * 1024;

  /// Most calls a batch may carry.
  pub(crate) const MAXIMUM_BATCH_CALLS: usize = 256;

  /// A batch's parts finished and not yet sent, past which its calls wait to hand theirs over.
  const BATCH_PARTS_IN_FLIGHT: usize = 8;

  /// A batch's calls read at once. Each holds its whole answer until its part is handed over, so with the parts waiting
  /// this bounds what a batch holds however slowly its parts are taken, as the browser's six connections did before.
  const BATCH_CALLS_AT_ONCE: usize = 8;

  /// What a batch's body is served as: parts, as `TransportPart` frames them.
  pub(crate) const PARTS: &'static str = "application/vnd.xrf.transport-parts";

  /// What a refusal is served as: its message, as a JSON string.
  const REFUSAL: &'static str = "application/json";

  /// Seconds a browser may keep a preflight's answer; Chromium holds none longer than two hours.
  const PREFLIGHT_MAXIMUM_AGE: &'static str = "7200";

  const ALLOWED_METHODS: &'static str = "POST";

  const ALLOWED_HEADERS: &'static str = "authorization, content-type";

  const ACCESS_CONTROL_REQUEST_PRIVATE_NETWORK: HeaderName =
    HeaderName::from_static("access-control-request-private-network");

  const ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK: HeaderName =
    HeaderName::from_static("access-control-allow-private-network");

  pub(crate) fn new(routes: TransportRoutes<C>, origins: TransportOrigins, token: TransportToken, context: C) -> Self {
    Self {
      routes,
      origins,
      token,
      context,
    }
  }

  /// Binds a loopback port the system picks, and answers where it is with the future serving it.
  ///
  /// Bound here rather than in the future, so the address is known before anything is spawned; the future accepts
  /// connections for as long as it is polled, and needs a tokio runtime.
  pub(crate) fn bind(self) -> io::Result<(SocketAddr, impl Future<Output = ()> + Send + 'static)> {
    let listener: StdTcpListener = StdTcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;

    listener.set_nonblocking(true)?;

    let address: SocketAddr = listener.local_addr()?;
    let server: Arc<Self> = Arc::new(self);

    Ok((address, server.serve(listener)))
  }

  async fn serve(self: Arc<Self>, listener: StdTcpListener) {
    let listener: TcpListener = match TcpListener::from_std(listener) {
      Ok(listener) => listener,
      Err(error) => {
        log::error!("Transport could not listen: {error}");

        return;
      }
    };

    log::info!(
      "Transport serving {} routes for {}",
      self.routes.len(),
      self.origins.list().join(", ")
    );

    loop {
      match listener.accept().await {
        Ok((stream, _)) => {
          tokio::spawn(Arc::clone(&self).connect(stream));
        }
        Err(error) => log::warn!("Transport failed to accept a connection: {error}"),
      }
    }
  }

  async fn connect(self: Arc<Self>, stream: TcpStream) {
    let service = service_fn(move |request: Request<Incoming>| {
      let server: Arc<Self> = Arc::clone(&self);

      async move { Ok::<Response<TransportBody>, Infallible>(server.answer(request).await) }
    });

    if let Err(error) = http1::Builder::new()
      .keep_alive(true)
      .serve_connection(TokioIo::new(stream), service)
      .await
    {
      log::debug!("Transport connection ended: {error}");
    }
  }

  async fn answer(self: &Arc<Self>, request: Request<Incoming>) -> Response<TransportBody> {
    // A browser names the page on every cross-origin request; nothing else is let through, and it is told so without
    // the headers that would let a page read why.
    let origin: Option<HeaderValue> = request
      .headers()
      .get(header::ORIGIN)
      .filter(|origin| origin.to_str().is_ok_and(|origin| self.origins.allows(origin)))
      .cloned();

    let Some(origin) = origin else {
      return Self::refuse(&TransportRefusal::Forbidden);
    };

    let mut response: Response<TransportBody> = if request.method() == Method::OPTIONS {
      Self::preflight(request.headers())
    } else {
      self
        .dispatch(request)
        .await
        .unwrap_or_else(|refusal| Self::refuse(&refusal))
    };

    response
      .headers_mut()
      .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

    response
  }

  async fn dispatch(self: &Arc<Self>, request: Request<Incoming>) -> Result<Response<TransportBody>, TransportRefusal> {
    if !self.token.is_authorizing(request.headers().get(header::AUTHORIZATION)) {
      return Err(TransportRefusal::Unauthorized);
    }

    if request.method() != Method::POST {
      return Err(TransportRefusal::MethodNotAllowed);
    }

    let path: &str = request.uri().path();

    if path.strip_prefix('/').unwrap_or(path) == Self::BATCH_PATH {
      let body: Bytes = Self::read_body(request, Self::MAXIMUM_BATCH_BODY_SIZE).await?;

      return Ok(self.batch(Self::read_batch(&body)?));
    }

    let route: &TransportRoute<C> = self.routes.get(path).ok_or(TransportRefusal::NotFound)?;
    let body: Bytes = Self::read_body(request, Self::MAXIMUM_BODY_SIZE).await?;

    route.call(self.context.clone(), body).await.map(Self::deliver)
  }

  async fn read_body(request: Request<Incoming>, limit: usize) -> Result<Bytes, TransportRefusal> {
    Ok(
      Limited::new(request.into_body(), limit)
        .collect()
        .await
        .map_err(|error| TransportRefusal::BadRequest(format!("The request body could not be read: {error}")))?
        .to_bytes(),
    )
  }

  fn read_batch(body: &[u8]) -> Result<Vec<TransportBatchCall>, TransportRefusal> {
    let calls: Vec<TransportBatchCall> = serde_json::from_slice(body)
      .map_err(|error| TransportRefusal::BadRequest(format!("The batch's calls do not read: {error}")))?;

    if calls.len() > Self::MAXIMUM_BATCH_CALLS {
      return Err(TransportRefusal::BadRequest(format!(
        "A batch carries at most {} calls, not {}",
        Self::MAXIMUM_BATCH_CALLS,
        calls.len()
      )));
    }

    Ok(calls)
  }

  /// Answers a batch: its calls read a few at a time, each answered by its part as it finishes, so one slow read holds
  /// up none of the others. A call refused answers its refusal as its part; the batch itself is answered `200`.
  fn batch(self: &Arc<Self>, calls: Vec<TransportBatchCall>) -> Response<TransportBody> {
    let (sender, parts): (Sender<TransportPart>, Receiver<TransportPart>) = mpsc::channel(Self::BATCH_PARTS_IN_FLIGHT);
    let reading: Arc<Semaphore> = Arc::new(Semaphore::new(Self::BATCH_CALLS_AT_ONCE));

    // At most `MAXIMUM_BATCH_CALLS` of them, so each index is a part's own.
    for (index, call) in (0_u32..).zip(calls) {
      let server: Arc<Self> = Arc::clone(self);
      let sender: Sender<TransportPart> = sender.clone();
      let reading: Arc<Semaphore> = Arc::clone(&reading);

      tokio::spawn(async move {
        // Held until the part is handed over; a call whose caller stopped reading is not read at all.
        let Ok(_permit) = reading.acquire_owned().await else {
          return;
        };

        if sender.is_closed() {
          return;
        }

        let answered: Result<TransportPart, TransportRefusal> = server
          .call(call)
          .await
          .and_then(|TransportAnswer { bytes, media_type }| {
            TransportPart::new(index, StatusCode::OK.as_u16(), media_type, Bytes::from(bytes))
          })
          .or_else(|refusal| {
            TransportPart::new(
              index,
              refusal.get_status().as_u16(),
              Self::REFUSAL,
              Bytes::from(Self::to_refusal_message(&refusal)),
            )
          });

        // A refusal past what a part carries leaves the call unanswered, which the reader tells as the batch ending
        // without it; a send that fails means the caller stopped reading.
        if let Ok(part) = answered {
          let _ = sender.send(part).await;
        }
      });
    }

    Self::respond(
      StatusCode::OK,
      TransportBody::parts(parts),
      Some(HeaderValue::from_static(Self::PARTS)),
    )
  }

  /// One call of a batch, answered as its route answers a request of its own.
  async fn call(&self, call: TransportBatchCall) -> Result<TransportAnswer, TransportRefusal> {
    let TransportBatchCall { route, args } = call;
    let route: &TransportRoute<C> = self.routes.get(&route).ok_or(TransportRefusal::NotFound)?;

    route
      .call(self.context.clone(), Bytes::copy_from_slice(args.get().as_bytes()))
      .await
  }

  /// The answer to a preflight, which carries no token: a browser sends none before it knows it may.
  fn preflight(requested: &HeaderMap) -> Response<TransportBody> {
    let mut response: Response<TransportBody> =
      Self::respond(StatusCode::NO_CONTENT, TransportBody::whole(Bytes::new()), None);
    let headers: &mut HeaderMap = response.headers_mut();

    headers.insert(
      header::ACCESS_CONTROL_ALLOW_METHODS,
      HeaderValue::from_static(Self::ALLOWED_METHODS),
    );
    headers.insert(
      header::ACCESS_CONTROL_ALLOW_HEADERS,
      HeaderValue::from_static(Self::ALLOWED_HEADERS),
    );
    headers.insert(
      header::ACCESS_CONTROL_MAX_AGE,
      HeaderValue::from_static(Self::PREFLIGHT_MAXIMUM_AGE),
    );

    // Asked by a page Chromium counts as a less private network than loopback; the application's own never is, but
    // answering costs nothing where it is.
    if requested.contains_key(Self::ACCESS_CONTROL_REQUEST_PRIVATE_NETWORK) {
      headers.insert(
        Self::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK,
        HeaderValue::from_static("true"),
      );
    }

    response
  }

  fn deliver(answer: TransportAnswer) -> Response<TransportBody> {
    Self::respond(
      StatusCode::OK,
      TransportBody::whole(Bytes::from(answer.bytes)),
      Some(HeaderValue::from_static(answer.media_type)),
    )
  }

  /// A refusal, its message as the JSON string an IPC command rejects with.
  fn refuse(refusal: &TransportRefusal) -> Response<TransportBody> {
    Self::respond(
      refusal.get_status(),
      TransportBody::whole(Bytes::from(Self::to_refusal_message(refusal))),
      Some(HeaderValue::from_static(Self::REFUSAL)),
    )
  }

  fn to_refusal_message(refusal: &TransportRefusal) -> Vec<u8> {
    serde_json::to_vec(&refusal.get_message()).unwrap_or_default()
  }

  fn respond(status: StatusCode, body: TransportBody, media_type: Option<HeaderValue>) -> Response<TransportBody> {
    let mut response: Response<TransportBody> = Response::new(body);

    *response.status_mut() = status;

    let headers: &mut HeaderMap = response.headers_mut();

    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::VARY, HeaderValue::from_static("Origin"));

    if let Some(media_type) = media_type {
      headers.insert(header::CONTENT_TYPE, media_type);
    }

    response
  }
}
