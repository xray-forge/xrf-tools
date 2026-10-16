//! The first lines of the log: which build ran, on what machine, and with which environment variables.

use std::env;
use std::path::PathBuf;

use xrf_build_info::{BuildInfo, build_info};
use xrf_env_flags::{EnvFlagReading, EnvFlags};
use xrf_utils::format_path;

use crate::plugins::system::diagnostics::host_info::HostInfo;

/// Logs the build, the machine, the process and the set environment variables, one line each.
pub fn log_startup() {
  log::info!("Build: {}", describe_build(&build_info!()));
  log::info!("Host: {}", describe_host(&HostInfo::read()));
  log::info!("Process: {}", describe_process());

  let set: Vec<EnvFlagReading> = EnvFlags::list_readings()
    .into_iter()
    .filter(EnvFlagReading::is_set)
    .collect();

  match set.is_empty() {
    true => log::info!("Environment: no variable set"),
    false => log::info!("Environment: {}", describe_flags(&set)),
  }
}

/// The build in one line: version and kind, commit, build time, target and optimisation.
fn describe_build(build: &BuildInfo) -> String {
  let mut parts: Vec<String> = vec![format!("xrf-app {} ({})", build.version, build.kind.as_str())];

  if let Some(commit) = build.short_commit() {
    let reference: String = build.reference.map(|it| format!(" on {it}")).unwrap_or_default();
    let dirty: &str = if build.is_dirty { ", dirty" } else { "" };

    parts.push(format!("commit {commit}{reference}{dirty}"));
  }

  // The value already starts with the compiler's name: `rustc 1.99.0 (...)`.
  parts.extend(build.rustc.map(String::from));

  for (label, value) in [
    ("built", build.built_at),
    ("target", build.target),
    ("profile", build.profile),
    ("optimization", build.optimization),
    ("workflow run", build.run_id),
  ] {
    if let Some(value) = value {
      parts.push(format!("{label} {value}"));
    }
  }

  parts.join(", ")
}

/// The machine in one line: system, processors, memory and the webview version.
fn describe_host(host: &HostInfo) -> String {
  let system: String = [host.os_name.as_deref(), host.os_version.as_deref()]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>()
    .join(" ");
  let cores: String = host
    .physical_core_count
    .map(|it| format!(" / {it} physical"))
    .unwrap_or_default();
  [
    if system.is_empty() {
      String::from("unknown system")
    } else {
      system
    },
    format!("kernel {}", host.kernel_version.as_deref().unwrap_or("unknown")),
    host.arch.clone(),
    format!("{} logical{cores} processors", host.cpu_count),
    format!("{:.1} GiB memory", host.total_memory as f64 / f64::from(1u32 << 30)),
    format!("Tauri {}", host.tauri_version),
    format!("webview {}", host.webview_version.as_deref().unwrap_or("unknown")),
  ]
  .join(", ")
}

/// The process in one line: its ID, the executable and the working directory.
fn describe_process() -> String {
  let describe =
    |path: std::io::Result<PathBuf>| path.map_or_else(|_| String::from("unknown"), |it| format_path(&it).to_string());
  let executable: String = describe(env::current_exe());
  let directory: String = describe(env::current_dir());

  format!(
    "{}, executable '{executable}', working directory '{directory}'",
    std::process::id()
  )
}

/// The set variables as `NAME=value`, with invalid values marked.
fn describe_flags(set: &[EnvFlagReading]) -> String {
  set
    .iter()
    .map(|reading| {
      let value: &str = reading.value.as_deref().unwrap_or_default();

      match reading.is_refused() {
        true => format!(
          "{}={value:?} (not valid, the default applies: {})",
          reading.name, reading.default
        ),
        false => format!("{}={value:?}", reading.name),
      }
    })
    .collect::<Vec<String>>()
    .join("; ")
}
