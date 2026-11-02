use std::collections::BTreeSet;

use tauri::AppHandle;
use tauri::plugin::Plugin;
use tauri::test::MockRuntime;
use xrf_app_core::jobs::JobKind;
use xrf_app_core::transport::TransportRoutes;

use crate::ipc::registry::transport_routes;
use crate::plugins::domain_plugins;

include!(concat!(env!("OUT_DIR"), "/inline_plugins.rs"));

/// The plugins the application sets up are exactly the inline plugins the build script declared, so no crate's
/// commands are permitted with nothing to dispatch them and none dispatches what nothing permits.
#[test]
fn every_inline_plugin_is_set_up_and_nothing_else() {
  let set_up: BTreeSet<&str> = domain_plugins::<MockRuntime>()
    .iter()
    .map(|plugin| plugin.name())
    .collect();

  assert_eq!(set_up, INLINE_PLUGINS.iter().copied().collect());
}

/// The frontend is generated a surface for exactly the inline plugins, so no plugin goes without its bindings.
#[cfg(feature = "typescript-bindings")]
#[test]
fn every_inline_plugin_is_mirrored_and_nothing_else() {
  let mirrored: BTreeSet<&str> = crate::ipc::bindings::command_surfaces::command_surfaces::<MockRuntime>()
    .iter()
    .map(|surface| surface.name)
    .collect();

  assert_eq!(mirrored, INLINE_PLUGINS.iter().copied().collect());
}

/// Every kind of job is its own, and spelled under its own domain twice over: its wire value leads with the domain's
/// plugin name, and its constant with the same name upper-cased, a hyphen an underscore, so no two domains' kinds or
/// constants can meet.
#[test]
fn every_job_kind_is_unique_and_carries_its_domains_prefix() {
  macro_rules! collect_job_kinds {
    ($($krate:ident),*) => {
      [$($krate::ipc::list_job_kinds(),)*].concat()
    };
  }

  let kinds: Vec<(&str, JobKind)> = for_each_plugin_crate!(collect_job_kinds);
  let wires: BTreeSet<&str> = kinds.iter().map(|(_, kind)| kind.as_str()).collect();

  assert!(!kinds.is_empty());
  assert_eq!(wires.len(), kinds.len(), "a job kind is declared twice: {kinds:?}");

  for (constant, kind) in &kinds {
    let (domain, operation) = kind.as_str().split_once('.').expect("a kind to lead with its domain");

    assert!(!operation.is_empty(), "'{kind}' names no operation");
    assert!(
      constant.starts_with(&format!("{}_JOB_", domain.to_uppercase().replace('-', "_"))),
      "'{constant}' does not lead with its domain '{domain}'"
    );
  }
}

#[test]
fn every_bulk_route_is_served_at_its_plugin_and_name() {
  let routes: TransportRoutes<AppHandle> = transport_routes();
  let paths: [&str; 8] = [
    "archives/read_entry",
    "archives/read_texture",
    "assets/read_asset",
    "assets/read_texture",
    "levels/read_screenshot",
    "sprite-equipment/read_preview",
    "textures/read_candidate",
    "textures/read_texels",
  ];

  assert_eq!(routes.len(), paths.len());

  for path in paths {
    assert!(routes.get(path).is_some(), "'{path}' is not served");
  }
}

#[test]
fn the_page_is_granted_no_file_command_but_exists() {
  let capability: serde_json::Value =
    serde_json::from_str(include_str!("../../../capabilities/default.json")).expect("the default capability to parse");
  let granted: Vec<&str> = capability["permissions"]
    .as_array()
    .expect("the capability to list its permissions")
    .iter()
    .filter_map(|permission| permission.as_str().or_else(|| permission["identifier"].as_str()))
    .filter(|identifier| identifier.starts_with("fs:"))
    .collect();

  assert_eq!(granted, ["fs:allow-exists", "fs:scope"]);
}
