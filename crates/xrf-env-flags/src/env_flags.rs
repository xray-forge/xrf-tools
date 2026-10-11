use std::fmt::Write;

use crate::env_flag::EnvFlag;
use crate::env_flag_kind::EnvFlagKind;
use crate::env_flag_reading::EnvFlagReading;
use crate::env_flag_value::EnvFlagValue;

/// The workspace's environment variables: each a constant read through, all in `ALL` for the document and the reports.
pub struct EnvFlags;

impl EnvFlags {
  pub const RENDER_BACKEND: EnvFlag = EnvFlag {
    name: "XRF_RENDER_BACKEND",
    kind: EnvFlagKind::Runtime,
    value: EnvFlagValue::Choice(&["d3d12", "vulkan"]),
    default: "the settings' choice",
    readers: &["xrf-app"],
    condition: None,
    description: "The graphics API the renderer starts on, over the settings' choice; one that cannot start falls back to \
                  the other.",
  };

  pub const RENDER_RAY_QUERY: EnvFlag = EnvFlag {
    name: "XRF_RENDER_RAY_QUERY",
    kind: EnvFlagKind::Runtime,
    value: EnvFlagValue::Switch,
    default: "on where the GPU has ray queries",
    readers: &["xrf-app"],
    condition: None,
    description: "Off leaves hardware ray queries unused, so the traced effects fall back to their untraced forms.",
  };

  pub const APP_EXTENSIONS_DIR: EnvFlag = EnvFlag {
    name: "XRF_APP_EXTENSIONS_DIR",
    kind: EnvFlagKind::Runtime,
    value: EnvFlagValue::Path,
    default: "`bin/xrf-app/extensions/unpacked` in the checkout",
    readers: &["xrf-app"],
    condition: Some("debug builds on Windows"),
    description: "The directory unpacked browser extensions are loaded into the webview from, every entry a folder with \
                  a `manifest.json`.",
  };

  pub const RUST_LOG: EnvFlag = EnvFlag {
    name: "RUST_LOG",
    kind: EnvFlagKind::PassThrough,
    value: EnvFlagValue::Text,
    default: "`info` for xrf-app (`debug` in a debug build), `error` for xrf-cli (`warn` in a debug build)",
    readers: &["xrf-app", "xrf-cli"],
    condition: None,
    description: "The log's filter, in `env_logger`'s syntax, such as `info,wgpu_hal=warn`.",
  };

  pub const WGPU_VALIDATION: EnvFlag = Self::wgpu_switch(
    "WGPU_VALIDATION",
    "wgpu's validation of every call, as debug builds have it.",
  );

  pub const WGPU_DEBUG: EnvFlag = Self::wgpu_switch(
    "WGPU_DEBUG",
    "wgpu's debug labels and markers on GPU objects, for graphics debuggers.",
  );

  pub const WGPU_GPU_BASED_VALIDATION: EnvFlag = Self::wgpu_switch(
    "WGPU_GPU_BASED_VALIDATION",
    "The driver's GPU-based validation, far slower; with `WGPU_VALIDATION` on.",
  );

  pub const WGPU_VALIDATION_INDIRECT_CALL: EnvFlag = Self::wgpu_switch(
    "WGPU_VALIDATION_INDIRECT_CALL",
    "wgpu's validation of indirect draw and dispatch arguments.",
  );

  pub const WGPU_DISCARD_HAL_LABELS: EnvFlag = Self::wgpu_switch(
    "WGPU_DISCARD_HAL_LABELS",
    "Labels left off the graphics API's own objects.",
  );

  pub const WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER: EnvFlag = Self::wgpu_switch(
    "WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER",
    "Adapters wgpu does not consider compliant offered anyway.",
  );

  pub const WGPU_STRICT_WEBGPU_COMPLIANCE: EnvFlag =
    Self::wgpu_switch("WGPU_STRICT_WEBGPU_COMPLIANCE", "WebGPU's limits enforced strictly.");

  pub const WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: EnvFlag = Self::webview2(
    "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
    "Chromium switches the webview starts with, such as `--auto-open-devtools-for-tabs`.",
  );

  pub const WEBVIEW2_BROWSER_EXECUTABLE_FOLDER: EnvFlag = Self::webview2(
    "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER",
    "A fixed-version WebView2 runtime to use instead of the installed one.",
  );

  pub const WEBVIEW2_RELEASE_CHANNEL_PREFERENCE: EnvFlag = Self::webview2(
    "WEBVIEW2_RELEASE_CHANNEL_PREFERENCE",
    "`1` looks for the least stable installed Edge channel first, rather than the stable runtime.",
  );

  pub const WEBVIEW2_USER_DATA_FOLDER: EnvFlag = Self::webview2(
    "WEBVIEW2_USER_DATA_FOLDER",
    "The webview's profile directory instead of the application's.",
  );

  pub const BUILD_KIND: EnvFlag = EnvFlag {
    name: "XRF_BUILD_KIND",
    kind: EnvFlagKind::Build,
    value: EnvFlagValue::Choice(&["development", "optimized"]),
    default: "`local`",
    readers: &["xrf-build-info"],
    condition: None,
    description: "What the build reports itself as, matched exactly; a label, not a profile: CI sets it.",
  };

  pub const BUILD_TIMESTAMP: EnvFlag = EnvFlag {
    name: "XRF_BUILD_TIMESTAMP",
    kind: EnvFlagKind::Build,
    value: EnvFlagValue::Text,
    default: "the time the build script runs",
    readers: &["xrf-build-info"],
    condition: None,
    description: "When the build reports it was made, copied as given (RFC 3339); empty leaves it out.",
  };

  pub const DLSS_SDK: EnvFlag = Self::sdk(
    "DLSS_SDK",
    "`.sdks/dlss`",
    "NVIDIA's DLSS SDK, for a build with the `dlss` feature.",
  );

  pub const VULKAN_SDK: EnvFlag = Self::sdk(
    "VULKAN_SDK",
    "`.sdks/vulkan-headers`",
    "The Vulkan headers DLSS's bindings are generated against.",
  );

  pub const LIBCLANG_PATH: EnvFlag = Self::sdk(
    "LIBCLANG_PATH",
    "`.sdks/libclang` on Windows, the system's elsewhere",
    "The libclang `bindgen` generates DLSS's bindings with.",
  );

  pub const DLSS_RUNTIME: EnvFlag = EnvFlag {
    name: "XRF_DLSS_RUNTIME",
    kind: EnvFlagKind::Internal,
    value: EnvFlagValue::Path,
    default: "set by the build script",
    readers: &["xrf-dlss"],
    condition: Some("builds with the `dlss` feature"),
    description: "The SDK's runtime directory, compiled in as the last place the DLSS library is looked for.",
  };

  /// Every variable, in the document's order.
  pub const ALL: &'static [EnvFlag] = &[
    Self::RENDER_BACKEND,
    Self::RENDER_RAY_QUERY,
    Self::APP_EXTENSIONS_DIR,
    Self::RUST_LOG,
    Self::WGPU_VALIDATION,
    Self::WGPU_DEBUG,
    Self::WGPU_GPU_BASED_VALIDATION,
    Self::WGPU_VALIDATION_INDIRECT_CALL,
    Self::WGPU_DISCARD_HAL_LABELS,
    Self::WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
    Self::WGPU_STRICT_WEBGPU_COMPLIANCE,
    Self::WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS,
    Self::WEBVIEW2_BROWSER_EXECUTABLE_FOLDER,
    Self::WEBVIEW2_RELEASE_CHANNEL_PREFERENCE,
    Self::WEBVIEW2_USER_DATA_FOLDER,
    Self::BUILD_KIND,
    Self::BUILD_TIMESTAMP,
    Self::DLSS_SDK,
    Self::VULKAN_SDK,
    Self::LIBCLANG_PATH,
    Self::DLSS_RUNTIME,
  ];

  /// The document's sections: each kind, with what it says of them.
  const SECTIONS: [(EnvFlagKind, &'static str, &'static str); 4] = [
    (
      EnvFlagKind::Runtime,
      "Runtime",
      "Read by a running binary: set them in the environment that starts it.",
    ),
    (
      EnvFlagKind::PassThrough,
      "Passed through",
      "Other libraries' own, which they read themselves; the application reports them with its own at startup.",
    ),
    (
      EnvFlagKind::Build,
      "Build",
      "Read while the workspace builds, by a build script or CI.",
    ),
    (
      EnvFlagKind::Internal,
      "Internal",
      "Set by a build script for the compiler, never by hand; listed so the name is not reused.",
    ),
  ];

  /// How a running binary finds every variable it reads, set or not, in `ALL`'s order.
  pub fn list_readings() -> Vec<EnvFlagReading> {
    Self::ALL
      .iter()
      .filter(|flag| flag.kind.is_read_at_run())
      .map(EnvFlag::to_reading)
      .collect()
  }

  /// The variables `reader` reads at run, for its help.
  pub fn list_read_by(reader: &str) -> Vec<&'static EnvFlag> {
    Self::ALL
      .iter()
      .filter(|flag| flag.kind.is_read_at_run() && flag.readers.contains(&reader))
      .collect()
  }

  /// The document of every variable, as `ENVIRONMENT.md` holds it.
  pub fn to_markdown() -> String {
    let mut document: String = String::from(
      "# Environment variables\n\n\
       Every environment variable the workspace reads or honours. Generated from `crates/xrf-env-flags` \
       (`EnvFlags::ALL`); edit the table there, and its test rewrites this file.\n\n\
       A runtime switch takes `1`, `true`, `on` or `yes` for on and `0`, `false`, `off` or `no` for off, and a runtime \
       choice takes its names in any case; one set to what it does not take is warned of in the log, and its default \
       applies. Passed-through and build variables are read as their readers read them, as each says. The application logs every variable set at startup, and lists them in Settings, About.\n",
    );

    for (kind, title, summary) in Self::SECTIONS {
      let _ = write!(document, "\n## {title}\n\n{summary}\n");

      for flag in Self::ALL.iter().filter(|flag| flag.kind == kind) {
        let _ = write!(document, "\n### `{}`\n\n{}\n\n", flag.name, flag.description);
        let _ = writeln!(document, "- Takes: {}.", flag.value.describe());
        let _ = writeln!(document, "- Unset: {}.", flag.default);

        let readers: Vec<String> = flag.readers.iter().map(|reader| format!("`{reader}`")).collect();
        let _ = writeln!(document, "- Read by: {}.", readers.join(", "));

        if let Some(condition) = flag.condition {
          let _ = writeln!(document, "- Only in: {condition}.");
        }
      }
    }

    document
  }

  /// One of wgpu's instance switches, which it reads itself where the renderer makes its instance.
  const fn wgpu_switch(name: &'static str, description: &'static str) -> EnvFlag {
    EnvFlag {
      name,
      kind: EnvFlagKind::PassThrough,
      value: EnvFlagValue::ZeroOffSwitch,
      default: "as the build has it: wgpu turns validation and debug labels on in a debug build",
      readers: &["xrf-app"],
      condition: None,
      description,
    }
  }

  /// One of the WebView2 runtime's variables, which it reads itself where the window's webview starts.
  const fn webview2(name: &'static str, description: &'static str) -> EnvFlag {
    EnvFlag {
      name,
      kind: EnvFlagKind::PassThrough,
      value: EnvFlagValue::Text,
      default: "the runtime's own",
      readers: &["xrf-app"],
      condition: Some("Windows"),
      description,
    }
  }

  /// A build input `.cargo/config.toml` points at the fetched SDKs, which one set in the environment overrides.
  const fn sdk(name: &'static str, default: &'static str, description: &'static str) -> EnvFlag {
    EnvFlag {
      name,
      kind: EnvFlagKind::Build,
      value: EnvFlagValue::Path,
      default,
      readers: &["xrf-dlss"],
      condition: Some("builds with the `dlss` feature"),
      description,
    }
  }
}
