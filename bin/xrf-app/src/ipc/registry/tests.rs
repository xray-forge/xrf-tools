use serde::Deserialize;
use serde_json::json;
use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody, Response};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets};
use tauri::webview::InvokeRequest;
use tauri::{App, AppHandle, WebviewWindow, WebviewWindowBuilder};

use crate::core::transport::{TransportAnswer, TransportRoutes};
use crate::core::types::TauriResult;
use crate::ipc::registry::transport_routes;

/// A domain declaring every kind of entry, since no real domain declares a raw command today.
mod fixture_registry {
  define_runtime_domains! {
    fixture => "fixture" {
      ping => crate::ipc::registry::tests::fixture_ping,
    }
    @raw {
      read_bytes(length: "number") => crate::ipc::registry::tests::fixture_read_bytes,
    }
    @bulk {
      read_route(length: "number") => crate::ipc::registry::tests::fixture_read_route,
    }
  }
}

use fixture_registry::fixture;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "ping"))]
#[tauri::command(rename = "ping")]
fn fixture_ping() -> String {
  String::from("pong")
}

#[tauri::command(rename = "read_bytes")]
fn fixture_read_bytes(length: usize) -> Response {
  Response::new(vec![7; length])
}

#[derive(Deserialize)]
struct FixtureReadRouteRequest {
  length: usize,
}

async fn fixture_read_route(_app: AppHandle, request: FixtureReadRouteRequest) -> TauriResult<TransportAnswer> {
  Ok(TransportAnswer::octets(vec![7; request.length]))
}

/// Invokes `command` on a window of an application dispatching through the fixture domain's handler alone.
fn invoke_fixture(command: &str, body: InvokeBody) -> Result<InvokeResponseBody, serde_json::Value> {
  let app: App<MockRuntime> = mock_builder()
    .invoke_handler(fixture::handler())
    .build(mock_context(noop_assets()))
    .expect("mock application to build");
  let window: WebviewWindow<MockRuntime> = WebviewWindowBuilder::new(&app, "main", Default::default())
    .build()
    .expect("mock window to build");

  get_ipc_response(
    &window,
    InvokeRequest {
      cmd: command.to_string(),
      callback: CallbackFn(0),
      error: CallbackFn(1),
      url: "http://tauri.localhost".parse().expect("local url to parse"),
      body,
      headers: Default::default(),
      invoke_key: INVOKE_KEY.to_string(),
    },
  )
}

#[test]
fn every_bulk_route_is_served_at_its_plugin_and_name() {
  let routes: TransportRoutes<AppHandle> = transport_routes();
  let paths: [&str; 7] = [
    "assets/read_asset",
    "archives/read_texture",
    "textures/read_candidate",
    "textures/read_texture",
    "visuals/read_geometry",
    "visuals/read_motion",
    "visuals/read_texture",
  ];

  assert_eq!(routes.len(), paths.len());

  for path in paths {
    assert!(routes.get(path).is_some(), "'{path}' is not served");
  }
}

#[test]
fn a_raw_command_is_dispatched_and_answers_its_bytes_raw() {
  let answer: InvokeResponseBody =
    invoke_fixture("read_bytes", InvokeBody::Json(json!({ "length": 3 }))).expect("raw command to answer");

  assert!(
    matches!(&answer, InvokeResponseBody::Raw(bytes) if bytes == &[7, 7, 7]),
    "raw command answered {answer:?}"
  );
}

#[test]
fn a_typed_command_is_dispatched_beside_a_raw_one() {
  let answer: InvokeResponseBody = invoke_fixture("ping", InvokeBody::default()).expect("typed command to answer");

  assert_eq!(answer.deserialize::<String>().expect("answer to be a string"), "pong");
}

#[test]
fn a_bulk_route_is_served_by_the_transport_and_never_dispatched() {
  let routes: TransportRoutes<AppHandle> = fixture_registry::transport_routes();

  assert_eq!(routes.len(), 1);
  assert!(routes.get("fixture/read_route").is_some());
  assert!(routes.get("fixture/read_bytes").is_none());
  assert!(invoke_fixture("read_route", InvokeBody::Json(json!({ "length": 3 }))).is_err());
}

#[cfg(feature = "typescript-bindings")]
mod bindings {
  use std::fs;
  use std::path::PathBuf;

  use xrf_ipc_typescript::{IpcBindingsGenerator, IpcCommandSurface};
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::ipc::registry::tests::fixture;

  #[test]
  fn a_raw_command_is_listed_for_the_bindings_and_a_bulk_route_apart() {
    assert_eq!(fixture::RAW_COMMANDS, &[("read_bytes", &[("length", "number")][..])]);
    assert_eq!(fixture::BULK_ROUTES, &[("read_route", &[("length", "number")][..])]);
  }

  #[test]
  fn a_domain_with_raw_commands_is_generated_a_raw_module() {
    let output: PathBuf = build_absolute_generated_test_resource_path("registry-fixture-bindings");

    IpcBindingsGenerator::generate(
      &output,
      &[IpcCommandSurface::new(
        fixture::NAME,
        fixture::specta_builder::<tauri::Wry>(),
        fixture::RAW_COMMANDS,
        fixture::BULK_ROUTES,
      )],
    );

    let read = |name: &str| -> String {
      fs::read_to_string(output.join("commands").join(name)).unwrap_or_else(|error| panic!("{name}: {error}"))
    };
    let typed: String = read("fixture.ts");
    let raw: String = read("fixture-raw.ts");
    let bulk: String = read("fixture-bulk.ts");

    assert!(typed.contains("ping"), "{typed}");
    assert!(!typed.contains("read_bytes"), "{typed}");
    assert!(
      raw.contains("readBytes: (length: number): Promise<ArrayBuffer> =>\n    invokeRaw(\"plugin:fixture|read_bytes\", { length }),"),
      "{raw}"
    );
    assert!(bulk.contains("route: \"fixture/read_route\""), "{bulk}");
  }
}
