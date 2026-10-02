use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use hyper::header::HeaderValue;
use serde::Deserialize;

use crate::core::transport::{
  TransportAnswer, TransportOrigins, TransportRoute, TransportRoutes, TransportServer, TransportToken,
};
use crate::core::types::TauriResult;

const ORIGIN: &str = "http://tauri.localhost";

/// What a test route is handed, beside its arguments.
#[derive(Clone)]
struct TestContext {
  prefix: &'static str,
  /// The slow route's calls running now, and the most that ever ran at once.
  running: Arc<AtomicUsize>,
  most: Arc<AtomicUsize>,
}

/// The batch path, as a request names it.
fn batch_path() -> String {
  format!("/{}", TransportServer::<TestContext>::BATCH_PATH)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchoRequest {
  logical_path: String,
}

async fn echo(context: TestContext, request: EchoRequest) -> TauriResult<TransportAnswer> {
  Ok(TransportAnswer::octets(
    format!("{}{}", context.prefix, request.logical_path).into_bytes(),
  ))
}

async fn fail(_: TestContext, _: EchoRequest) -> TauriResult<TransportAnswer> {
  Err(String::from("The level sector session has changed or is closed"))
}

async fn empty(_: TestContext, _: EchoRequest) -> TauriResult<TransportAnswer> {
  Ok(TransportAnswer::octets(Vec::new()))
}

/// An echo that takes a while, other calls running meanwhile, counting how many run at once.
async fn slow(context: TestContext, request: EchoRequest) -> TauriResult<TransportAnswer> {
  let running: usize = context.running.fetch_add(1, Ordering::SeqCst) + 1;

  context.most.fetch_max(running, Ordering::SeqCst);

  for _ in 0..50 {
    tokio::task::yield_now().await;
  }

  context.running.fetch_sub(1, Ordering::SeqCst);

  echo(context, request).await
}

/// One answer as the client read it.
struct Answer {
  status: u16,
  headers: HashMap<String, String>,
  body: Vec<u8>,
}

impl Answer {
  fn header(&self, name: &str) -> Option<&str> {
    self.headers.get(name).map(String::as_str)
  }
}

/// A server on a real loopback port, serving the test routes, the token it expects and the most slow calls at once.
fn serve_counted() -> (SocketAddr, TransportToken, Arc<AtomicUsize>) {
  let token: TransportToken = TransportToken::generate().unwrap();
  let routes: TransportRoutes<TestContext> = TransportRoutes::new(vec![
    TransportRoute::new("assets", "echo", echo),
    TransportRoute::new("assets", "fail", fail),
    TransportRoute::new("assets", "empty", empty),
    TransportRoute::new("assets", "slow", slow),
  ]);
  let most: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
  let server: TransportServer<TestContext> = TransportServer::new(
    routes,
    TransportOrigins::new(vec![String::from(ORIGIN)]),
    token.clone(),
    TestContext {
      most: Arc::clone(&most),
      prefix: "read:",
      running: Arc::new(AtomicUsize::new(0)),
    },
  );
  let (address, serving) = server.bind().unwrap();

  tauri::async_runtime::spawn(serving);

  (address, token, most)
}

/// A server on a real loopback port, serving the test routes, and the token it expects.
fn serve() -> (SocketAddr, TransportToken) {
  let (address, token, _) = serve_counted();

  (address, token)
}

/// A batch body of one route called for each path.
fn to_batch(route: &str, paths: &[String]) -> String {
  let calls: Vec<String> = paths
    .iter()
    .map(|path| format!(r#"{{"route":"{route}","args":{{"logicalPath":"{path}"}}}}"#))
    .collect();

  format!("[{}]", calls.join(","))
}

/// Sends one request on an open connection and reads its whole answer, leaving the connection open.
fn send(stream: &mut TcpStream, method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Answer {
  let mut request: String = format!(
    "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\n",
    body.len()
  );

  for (name, value) in headers {
    request.push_str(&format!("{name}: {value}\r\n"));
  }

  request.push_str("\r\n");
  request.push_str(body);
  stream.write_all(request.as_bytes()).unwrap();

  let mut reader: BufReader<&mut TcpStream> = BufReader::new(stream);
  let mut line: String = String::new();

  reader.read_line(&mut line).unwrap();

  let status: u16 = line.split(' ').nth(1).unwrap().parse().unwrap();
  let mut headers: HashMap<String, String> = HashMap::new();

  loop {
    line.clear();
    reader.read_line(&mut line).unwrap();

    let header: &str = line.trim_end();

    if header.is_empty() {
      break;
    }

    let (name, value) = header.split_once(':').unwrap();

    headers.insert(name.to_ascii_lowercase(), value.trim().to_string());
  }

  let body: Vec<u8> = if headers.get("transfer-encoding").is_some_and(|it| it == "chunked") {
    read_chunked(&mut reader)
  } else {
    let length: usize = headers.get("content-length").map_or(0, |it| it.parse().unwrap());
    let mut body: Vec<u8> = vec![0; length];

    reader.read_exact(&mut body).unwrap();

    body
  };

  Answer { status, headers, body }
}

/// A chunked body, its chunks joined: what a batch is answered as, its parts sent as they come.
fn read_chunked(reader: &mut BufReader<&mut TcpStream>) -> Vec<u8> {
  let mut body: Vec<u8> = Vec::new();
  let mut line: String = String::new();

  loop {
    line.clear();
    reader.read_line(&mut line).unwrap();

    let size: usize = usize::from_str_radix(line.trim_end(), 16).unwrap();
    let mut chunk: Vec<u8> = vec![0; size + 2];

    reader.read_exact(&mut chunk).unwrap();

    if size == 0 {
      return body;
    }

    body.extend_from_slice(&chunk[..size]);
  }
}

/// One part of a batch's answer.
#[derive(Debug, PartialEq, Eq)]
struct Part {
  status: u16,
  media_type: String,
  body: Vec<u8>,
}

/// A batch's parts, by the index of the call each answers.
fn read_parts(mut body: &[u8]) -> HashMap<u32, Part> {
  let mut parts: HashMap<u32, Part> = HashMap::new();

  while !body.is_empty() {
    let index: u32 = u32::from_le_bytes(body[0..4].try_into().unwrap());
    let status: u16 = u16::from_le_bytes(body[4..6].try_into().unwrap());
    let media_length: usize = usize::from(u16::from_le_bytes(body[6..8].try_into().unwrap()));
    let length: usize = u32::from_le_bytes(body[8..12].try_into().unwrap()) as usize;
    let media_type: String = String::from_utf8(body[12..12 + media_length].to_vec()).unwrap();
    let start: usize = 12 + media_length;

    assert!(
      parts
        .insert(
          index,
          Part {
            status,
            media_type,
            body: body[start..start + length].to_vec(),
          },
        )
        .is_none(),
      "Call {index} answered twice"
    );
    body = &body[start + length..];
  }

  parts
}

fn connect(address: SocketAddr) -> TcpStream {
  TcpStream::connect(address).unwrap()
}

fn post(address: SocketAddr, path: &str, authorization: Option<&str>, body: &str) -> Answer {
  let mut headers: Vec<(&str, &str)> = vec![("Origin", ORIGIN), ("Content-Type", "application/json")];

  if let Some(authorization) = authorization {
    headers.push(("Authorization", authorization));
  }

  send(&mut connect(address), "POST", path, &headers, body)
}

fn bearer(token: &TransportToken) -> String {
  format!("Bearer {}", token.get_value())
}

#[test]
fn an_authorized_request_is_answered_by_its_route() {
  let (address, token) = serve();
  let answer: Answer = post(
    address,
    "/assets/echo",
    Some(&bearer(&token)),
    r#"{"logicalPath":"textures\\a.dds"}"#,
  );

  assert_eq!(answer.status, 200);
  assert_eq!(answer.body, b"read:textures\\a.dds");
  assert_eq!(answer.header("content-type"), Some("application/octet-stream"));
  assert_eq!(answer.header("cache-control"), Some("no-store"));
  assert_eq!(answer.header("access-control-allow-origin"), Some(ORIGIN));
}

#[test]
fn a_request_without_the_token_is_refused_readably() {
  let (address, token) = serve();

  for authorization in [None, Some("Bearer 00"), Some(token.get_value())] {
    let answer: Answer = post(address, "/assets/echo", authorization, r#"{"logicalPath":"a"}"#);

    assert_eq!(answer.status, 401);
    // The page's own origin, so the page can read why rather than see a network error.
    assert_eq!(answer.header("access-control-allow-origin"), Some(ORIGIN));
  }
}

#[test]
fn a_foreign_or_missing_origin_is_refused_without_cors_headers() {
  let (address, token) = serve();
  let authorization: String = bearer(&token);

  for origin in [Some("http://localhost:1420"), Some("https://example.com"), None] {
    let mut headers: Vec<(&str, &str)> = vec![("Authorization", authorization.as_str())];

    if let Some(origin) = origin {
      headers.push(("Origin", origin));
    }

    let answer: Answer = send(
      &mut connect(address),
      "POST",
      "/assets/echo",
      &headers,
      r#"{"logicalPath":"a"}"#,
    );

    assert_eq!(answer.status, 403);
    assert_eq!(answer.header("access-control-allow-origin"), None);
  }
}

#[test]
fn a_preflight_is_answered_without_a_token() {
  let (address, _) = serve();
  let answer: Answer = send(
    &mut connect(address),
    "OPTIONS",
    "/assets/echo",
    &[
      ("Origin", ORIGIN),
      ("Access-Control-Request-Method", "POST"),
      ("Access-Control-Request-Headers", "authorization, content-type"),
      ("Access-Control-Request-Private-Network", "true"),
    ],
    "",
  );

  assert_eq!(answer.status, 204);
  assert_eq!(answer.header("access-control-allow-origin"), Some(ORIGIN));
  assert_eq!(answer.header("access-control-allow-methods"), Some("POST"));
  assert_eq!(
    answer.header("access-control-allow-headers"),
    Some("authorization, content-type")
  );
  assert_eq!(answer.header("access-control-max-age"), Some("7200"));
  assert_eq!(answer.header("access-control-allow-private-network"), Some("true"));
}

#[test]
fn a_preflight_from_a_foreign_origin_is_refused() {
  let (address, _) = serve();
  let answer: Answer = send(
    &mut connect(address),
    "OPTIONS",
    "/assets/echo",
    &[
      ("Origin", "https://example.com"),
      ("Access-Control-Request-Method", "POST"),
    ],
    "",
  );

  assert_eq!(answer.status, 403);
  assert_eq!(answer.header("access-control-allow-origin"), None);
}

#[test]
fn only_declared_routes_are_served_and_only_by_post() {
  let (address, token) = serve();
  let authorization: String = bearer(&token);

  assert_eq!(post(address, "/assets/list", Some(&authorization), "{}").status, 404);
  assert_eq!(post(address, "/", Some(&authorization), "{}").status, 404);

  let answer: Answer = send(
    &mut connect(address),
    "GET",
    "/assets/echo",
    &[("Origin", ORIGIN), ("Authorization", authorization.as_str())],
    "",
  );

  assert_eq!(answer.status, 405);
}

#[test]
fn a_failed_route_answers_its_error_as_a_json_string() {
  let (address, token) = serve();
  let answer: Answer = post(address, "/assets/fail", Some(&bearer(&token)), r#"{"logicalPath":"a"}"#);

  assert_eq!(answer.status, 500);
  assert_eq!(answer.header("content-type"), Some("application/json"));
  assert_eq!(
    serde_json::from_slice::<String>(&answer.body).unwrap(),
    "The level sector session has changed or is closed"
  );
}

#[test]
fn arguments_that_do_not_read_are_a_bad_request() {
  let (address, token) = serve();
  let answer: Answer = post(address, "/assets/echo", Some(&bearer(&token)), r#"{"path":"a"}"#);

  assert_eq!(answer.status, 400);
  assert!(
    serde_json::from_slice::<String>(&answer.body)
      .unwrap()
      .starts_with("Arguments of 'assets/echo' do not read")
  );
}

#[test]
fn a_connection_is_kept_alive_across_requests() {
  let (address, token) = serve();
  let authorization: String = bearer(&token);
  let headers: [(&str, &str); 2] = [("Origin", ORIGIN), ("Authorization", authorization.as_str())];
  let mut stream: TcpStream = connect(address);

  assert_eq!(
    send(&mut stream, "POST", "/assets/echo", &headers, r#"{"logicalPath":"a"}"#).body,
    b"read:a"
  );
  assert_eq!(
    send(&mut stream, "POST", "/assets/echo", &headers, r#"{"logicalPath":"b"}"#).body,
    b"read:b"
  );
}

#[test]
fn a_batch_answers_each_call_by_its_part_and_a_refused_call_by_its_refusal() {
  let (address, token) = serve();
  let answer: Answer = post(
    address,
    &batch_path(),
    Some(&bearer(&token)),
    r#"[
      {"route":"assets/echo","args":{"logicalPath":"a"}},
      {"route":"assets/fail","args":{"logicalPath":"b"}},
      {"route":"assets/echo","args":{"path":"c"}},
      {"route":"assets/list","args":{}},
      {"route":"assets/echo","args":{"logicalPath":"textures\\d.dds"}}
    ]"#,
  );
  let parts: HashMap<u32, Part> = read_parts(&answer.body);
  let message = |index: u32| -> String { serde_json::from_slice(&parts[&index].body).unwrap() };

  assert_eq!(answer.status, 200);
  assert_eq!(
    answer.header("content-type"),
    Some(TransportServer::<TestContext>::PARTS)
  );
  assert_eq!(answer.header("access-control-allow-origin"), Some(ORIGIN));
  assert_eq!(parts.len(), 5);
  assert_eq!(
    parts[&0],
    Part {
      status: 200,
      media_type: String::from("application/octet-stream"),
      body: b"read:a".to_vec(),
    }
  );
  assert_eq!(parts[&1].status, 500);
  assert_eq!(parts[&1].media_type, "application/json");
  assert_eq!(message(1), "The level sector session has changed or is closed");
  assert_eq!(parts[&2].status, 400);
  assert!(message(2).starts_with("Arguments of 'assets/echo' do not read"));
  assert_eq!(parts[&3].status, 404);
  assert_eq!(parts[&4].body, b"read:textures\\d.dds");
}

#[test]
fn a_batch_of_nothing_is_answered_by_no_parts() {
  let (address, token) = serve();
  let answer: Answer = post(address, &batch_path(), Some(&bearer(&token)), "[]");

  assert_eq!(answer.status, 200);
  assert!(answer.body.is_empty());
}

#[test]
fn a_batch_is_refused_whole_without_the_token_or_with_calls_that_do_not_read() {
  let (address, token) = serve();
  let authorization: String = bearer(&token);
  let too_many: String = format!(
    "[{}]",
    vec![
      r#"{"route":"assets/echo","args":{"logicalPath":"a"}}"#;
      TransportServer::<TestContext>::MAXIMUM_BATCH_CALLS + 1
    ]
    .join(",")
  );

  assert_eq!(post(address, &batch_path(), None, "[]").status, 401);
  assert_eq!(
    post(address, &batch_path(), Some(&authorization), r#"{"route":"a"}"#).status,
    400
  );
  assert_eq!(
    post(address, &batch_path(), Some(&authorization), &too_many).status,
    400
  );
}

// Past the parts a batch keeps waiting, a call that finished waits to hand its part over rather than be dropped.
#[test]
fn a_batch_longer_than_the_parts_it_keeps_waiting_answers_every_call_and_reads_a_few_at_once() {
  let (address, token, most) = serve_counted();
  let paths: Vec<String> = (0..40).map(|index| format!("t{index}")).collect();
  let answer: Answer = post(
    address,
    &batch_path(),
    Some(&bearer(&token)),
    &to_batch("assets/slow", &paths),
  );
  let parts: HashMap<u32, Part> = read_parts(&answer.body);

  assert_eq!(parts.len(), 40);

  for (index, path) in (0_u32..).zip(&paths) {
    assert_eq!(parts[&index].body, format!("read:{path}").into_bytes());
  }

  assert!(
    most.load(Ordering::SeqCst) <= 8,
    "{} calls ran at once",
    most.load(Ordering::SeqCst)
  );
}

#[test]
fn a_call_answering_nothing_is_a_part_of_no_bytes() {
  let (address, token) = serve();
  let answer: Answer = post(
    address,
    &batch_path(),
    Some(&bearer(&token)),
    &to_batch("assets/empty", &[String::from("a")]),
  );
  let parts: HashMap<u32, Part> = read_parts(&answer.body);

  assert_eq!(parts[&0].status, 200);
  assert!(parts[&0].body.is_empty());
}

// A page that stops reading a batch, as one does that aborted every call of it, leaves the server serving.
#[test]
fn a_caller_leaving_a_batch_unread_leaves_the_server_serving() {
  let (address, token) = serve();
  let authorization: String = bearer(&token);
  let paths: Vec<String> = (0..200).map(|index| format!("{index}")).collect();
  let body: String = to_batch("assets/echo", &paths);
  let mut stream: TcpStream = connect(address);
  let request: String = format!(
    "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: {ORIGIN}\r\nAuthorization: {authorization}\r\nContent-Length: {}\r\n\r\n{body}",
    batch_path(),
    body.len()
  );

  stream.write_all(request.as_bytes()).unwrap();
  drop(stream);

  assert_eq!(
    post(address, "/assets/echo", Some(&authorization), r#"{"logicalPath":"a"}"#).body,
    b"read:a"
  );
}

#[test]
fn a_token_authorizes_only_itself_under_the_bearer_scheme() {
  let token: TransportToken = TransportToken::generate().unwrap();
  let other: TransportToken = TransportToken::generate().unwrap();
  let header = |value: String| HeaderValue::from_str(&value).unwrap();

  assert_eq!(token.get_value().len(), 64);
  assert_ne!(token.get_value(), other.get_value());
  assert!(token.is_authorizing(Some(&header(format!("Bearer {}", token.get_value())))));
  assert!(!token.is_authorizing(Some(&header(format!("Bearer {}", other.get_value())))));
  assert!(!token.is_authorizing(Some(&header(format!("Basic {}", token.get_value())))));
  assert!(!token.is_authorizing(Some(&header(format!("Bearer {}0", token.get_value())))));
  assert!(!token.is_authorizing(None));
}

#[test]
fn a_development_build_allows_its_dev_server_beside_the_bundled_origins() {
  let mut config: tauri::Config = serde_json::from_str(r#"{"identifier":"com.xrf.test"}"#).unwrap();

  config.build.dev_url = Some("http://localhost:1421/".parse().unwrap());

  let origins: TransportOrigins = TransportOrigins::of_config(&config);

  // Tests build without `custom-protocol`, which is what a development build is.
  assert!(origins.allows("http://localhost:1421"));
  assert!(!origins.allows("http://localhost:1420"));

  for bundled in TransportOrigins::BUNDLED {
    assert!(origins.allows(bundled));
  }
}

#[test]
#[should_panic(expected = "Transport route 'assets/echo' is declared twice")]
fn two_routes_at_one_path_are_refused() {
  TransportRoutes::<TestContext>::new(vec![
    TransportRoute::new("assets", "echo", echo),
    TransportRoute::new("assets", "echo", fail),
  ]);
}
