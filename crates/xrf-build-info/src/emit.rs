//! The build-script half: records what a binary should be able to say about itself.
//!
//! Values come from the environment first and Git second. Continuous integration knows things a checkout
//! cannot - which workflow run this was, and whether the build was meant for turnaround or for release -
//! while a developer machine is the only place a dirty tree exists.

use std::env;
use std::process::Command;

/// Write the build description into the compiling crate's environment.
///
/// Call from a binary's `build.rs`. Reading it back needs `build_info!` in that same crate, because
/// `env!` resolves where it is written rather than here.
pub fn emit() {
  // Any `rerun-if` directive replaces cargo's default of re-running whenever a package file changes, so
  // every input has to be named. Without the Git log a local commit would leave a stale hash embedded.
  for variable in [
    "XRF_BUILD_KIND",
    "XRF_BUILD_TIMESTAMP",
    "GITHUB_SHA",
    "GITHUB_REF_NAME",
    "GITHUB_RUN_ID",
    "CARGO_PROFILE_RELEASE_LTO",
    "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
  ] {
    println!("cargo:rerun-if-env-changed={variable}");
  }

  if let Some(git_dir) = git_output(&["rev-parse", "--absolute-git-dir"]) {
    println!("cargo:rerun-if-changed={git_dir}/HEAD");
    println!("cargo:rerun-if-changed={git_dir}/logs/HEAD");
  }

  emit_variable("XRF_BUILD_KIND", env::var("XRF_BUILD_KIND").ok());
  emit_variable(
    "XRF_BUILD_COMMIT",
    env::var("GITHUB_SHA")
      .ok()
      .or_else(|| git_output(&["rev-parse", "HEAD"])),
  );
  emit_variable(
    "XRF_BUILD_REF",
    env::var("GITHUB_REF_NAME")
      .ok()
      .or_else(|| git_output(&["rev-parse", "--abbrev-ref", "HEAD"])),
  );
  emit_variable("XRF_BUILD_DIRTY", Some(is_dirty().to_string()));
  // Supplied by CI so the recorded instant is the workflow's, and computed otherwise: two dirty builds
  // of one commit are otherwise indistinguishable, which is exactly the local case.
  emit_variable(
    "XRF_BUILD_TIMESTAMP",
    env::var("XRF_BUILD_TIMESTAMP").ok().or_else(|| Some(built_at())),
  );
  emit_variable("XRF_BUILD_TARGET", env::var("TARGET").ok());
  emit_variable("XRF_BUILD_RUSTC", rustc_version());
  emit_variable("XRF_BUILD_PROFILE", env::var("PROFILE").ok());
  emit_variable("XRF_BUILD_OPTIMIZATION", Some(optimization()));
  emit_variable("XRF_BUILD_RUN_ID", env::var("GITHUB_RUN_ID").ok());
}

/// Set a variable only when there is a value, so the reader sees absence rather than an empty string.
fn emit_variable(name: &str, value: Option<String>) {
  if let Some(value) = value.filter(|value| !value.is_empty()) {
    println!("cargo:rustc-env={name}={value}");
  }
}

/// How hard the compiler was asked to work, as cargo resolved it for this build.
///
/// Link-time optimisation and codegen units are not handed to build scripts, so they can only be named
/// when something overrode them in the environment. A build using the profile as committed says so
/// rather than guessing values it cannot see.
fn optimization() -> String {
  let level: String = env::var("OPT_LEVEL").unwrap_or_else(|_| String::from("unknown"));
  let lto: String = env::var("CARGO_PROFILE_RELEASE_LTO").unwrap_or_else(|_| String::from("profile default"));
  let units: String =
    env::var("CARGO_PROFILE_RELEASE_CODEGEN_UNITS").unwrap_or_else(|_| String::from("profile default"));

  format!("opt-level={level}, lto={lto}, codegen-units={units}")
}

fn rustc_version() -> Option<String> {
  let rustc: String = env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"));
  let output = Command::new(rustc).arg("--version").output().ok()?;

  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Whether the checkout carries changes that are in the binary but in no commit.
fn is_dirty() -> bool {
  git_output(&["status", "--porcelain"]).is_some_and(|status| !status.is_empty())
}

/// Run a Git command, treating any failure as an absent answer.
///
/// A build can legitimately happen outside a checkout - from a source archive, or in a container that
/// carries no Git - and none of that should fail a compile.
fn git_output(arguments: &[&str]) -> Option<String> {
  let output = Command::new("git").args(arguments).output().ok()?;

  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The current instant, to a whole second.
///
/// `Timestamp` renders RFC 3339 in UTC, and rounding drops the sub-second digits that would otherwise
/// make the value noisier than anything reading it needs.
fn built_at() -> String {
  jiff::Timestamp::now()
    .round(jiff::Unit::Second)
    .unwrap_or_else(|_| jiff::Timestamp::now())
    .to_string()
}
