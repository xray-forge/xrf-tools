use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener as StdTcpListener};
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};

use crate::core::transport::{
  TransportAnswer, TransportOrigins, TransportRefusal, TransportRoute, TransportRoutes, TransportToken,
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

      async move { Ok::<Response<Full<Bytes>>, Infallible>(server.answer(request).await) }
    });

    if let Err(error) = http1::Builder::new()
      .keep_alive(true)
      .serve_connection(TokioIo::new(stream), service)
      .await
    {
      log::debug!("Transport connection ended: {error}");
    }
  }

  async fn answer(&self, request: Request<Incoming>) -> Response<Full<Bytes>> {
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

    let mut response: Response<Full<Bytes>> = if request.method() == Method::OPTIONS {
      Self::preflight(request.headers())
    } else {
      match self.dispatch(request).await {
        Ok(answer) => Self::deliver(answer),
        Err(refusal) => Self::refuse(&refusal),
      }
    };

    response
      .headers_mut()
      .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

    response
  }

  async fn dispatch(&self, request: Request<Incoming>) -> Result<TransportAnswer, TransportRefusal> {
    if !self.token.is_authorizing(request.headers().get(header::AUTHORIZATION)) {
      return Err(TransportRefusal::Unauthorized);
    }

    if request.method() != Method::POST {
      return Err(TransportRefusal::MethodNotAllowed);
    }

    let route: &TransportRoute<C> = self
      .routes
      .get(request.uri().path())
      .ok_or(TransportRefusal::NotFound)?;
    let body: Bytes = Limited::new(request.into_body(), Self::MAXIMUM_BODY_SIZE)
      .collect()
      .await
      .map_err(|error| TransportRefusal::BadRequest(format!("The request body could not be read: {error}")))?
      .to_bytes();

    route.call(self.context.clone(), body).await
  }

  /// The answer to a preflight, which carries no token: a browser sends none before it knows it may.
  fn preflight(requested: &HeaderMap) -> Response<Full<Bytes>> {
    let mut response: Response<Full<Bytes>> = Self::respond(StatusCode::NO_CONTENT, Bytes::new(), None);
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

  fn deliver(answer: TransportAnswer) -> Response<Full<Bytes>> {
    Self::respond(
      StatusCode::OK,
      Bytes::from(answer.bytes),
      Some(HeaderValue::from_static(answer.media_type)),
    )
  }

  /// A refusal, its message as the JSON string an IPC command rejects with.
  fn refuse(refusal: &TransportRefusal) -> Response<Full<Bytes>> {
    let message: Vec<u8> = serde_json::to_vec(&refusal.get_message()).unwrap_or_default();

    Self::respond(
      refusal.get_status(),
      Bytes::from(message),
      Some(HeaderValue::from_static("application/json")),
    )
  }

  fn respond(status: StatusCode, body: Bytes, media_type: Option<HeaderValue>) -> Response<Full<Bytes>> {
    let mut response: Response<Full<Bytes>> = Response::new(Full::new(body));

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
