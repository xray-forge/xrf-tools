use std::fmt::Write;

use crate::choice_flag::ChoiceFlag;
use crate::env_flag::EnvFlag;
use crate::env_flag_kind::EnvFlagKind;
use crate::env_flag_reading::EnvFlagReading;
use crate::env_flag_value::EnvFlagValue;
use crate::path_flag::PathFlag;
use crate::switch_flag::SwitchFlag;
use crate::text_flag::TextFlag;

/// Every environment variable the workspace reads. Code reads each through its constant; `ALL` lists them all.
pub struct EnvFlags;

impl EnvFlags {
  pub const RENDER_BACKEND: ChoiceFlag = ChoiceFlag::new(EnvFlag {
    name: "XRF_RENDER_BACKEND",
    kind: EnvFlagKind::Runtime,
    value: EnvFlagValue::Choice(&["d3d12", "vulkan", "metal"]),
    default: "the API chosen in the settings",
    readers: &["xrf-app"],
    condition: None,
    description: "Starts the renderer on this graphics API instead of the one chosen in the settings. If it cannot \
                  start, the renderer falls back to the other APIs of the platform: D3D12 and Vulkan on Windows, Metal \
                  on macOS (experimental).",
  });

  pub const RENDER_RAY_QUERY: SwitchFlag = SwitchFlag::new(EnvFlag {
    name: "XRF_RENDER_RAY_QUERY",
    kind: EnvFlagKind::Runtime,
    value: EnvFlagValue::Switch,
    default: "on when the GPU supports ray queries",
    readers: &["xrf-app"],
    condition: None,
    description: "Set it off to stop the renderer using hardware ray queries. Traced effects then use their untraced \
                  versions.",
  });

  pub const APP_EXTENSIONS_DIR: PathFlag = PathFlag::new(EnvFlag {
    name: "XRF_APP_EXTENSIONS_DIR",
    kind: EnvFlagKind::Runtime,
    value: EnvFlagValue::Path,
    default: "`bin/xrf-app/extensions/unpacked` in the checkout",
    readers: &["xrf-app"],
    condition: Some("debug builds on Windows"),
    description: "Directory to load unpacked browser extensions into the webview from. Each entry must be a folder \
                  with a `manifest.json`.",
  });

  pub const RUST_LOG: TextFlag = TextFlag::new(EnvFlag {
    name: "RUST_LOG",
    kind: EnvFlagKind::PassThrough,
    value: EnvFlagValue::Text,
    default: "`info` for xrf-app (`debug` in a debug build), `error` for xrf-cli (`warn` in a debug build)",
    readers: &["xrf-app", "xrf-cli"],
    condition: None,
    description: "Log filter in `env_logger` syntax, for example `info,wgpu_hal=warn`.",
  });

  pub const WGPU_VALIDATION: SwitchFlag = Self::wgpu_switch(
    "WGPU_VALIDATION",
    "on in debug builds, off otherwise",
    "Turns wgpu's API validation on or off.",
  );

  pub const WGPU_DEBUG: SwitchFlag = Self::wgpu_switch(
    "WGPU_DEBUG",
    "on in debug builds, off otherwise",
    "Turns wgpu's debug labels and markers on GPU objects on or off, for graphics debuggers.",
  );

  pub const WGPU_GPU_BASED_VALIDATION: SwitchFlag = Self::wgpu_switch(
    "WGPU_GPU_BASED_VALIDATION",
    "off",
    "Turns on the driver's GPU-based validation. It is much slower and needs `WGPU_VALIDATION` on.",
  );

  pub const WGPU_VALIDATION_INDIRECT_CALL: SwitchFlag = Self::wgpu_switch(
    "WGPU_VALIDATION_INDIRECT_CALL",
    "on",
    "Turns wgpu's validation of indirect draw and dispatch arguments on or off.",
  );

  pub const WGPU_DISCARD_HAL_LABELS: SwitchFlag = Self::wgpu_switch(
    "WGPU_DISCARD_HAL_LABELS",
    "off",
    "When on, wgpu does not pass labels down to the graphics API's objects.",
  );

  pub const WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER: SwitchFlag = Self::wgpu_switch(
    "WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER",
    "off",
    "When on, wgpu also offers adapters it does not consider compliant.",
  );

  pub const WGPU_STRICT_WEBGPU_COMPLIANCE: SwitchFlag = Self::wgpu_switch(
    "WGPU_STRICT_WEBGPU_COMPLIANCE",
    "off",
    "When on, wgpu enforces WebGPU's limits strictly.",
  );

  pub const WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: TextFlag = Self::webview2(
    "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
    "Extra Chromium switches for the webview, for example `--auto-open-devtools-for-tabs`.",
  );

  pub const WEBVIEW2_BROWSER_EXECUTABLE_FOLDER: TextFlag = Self::webview2(
    "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER",
    "Folder of a fixed-version WebView2 runtime to use instead of the installed one.",
  );

  pub const WEBVIEW2_RELEASE_CHANNEL_PREFERENCE: TextFlag = Self::webview2(
    "WEBVIEW2_RELEASE_CHANNEL_PREFERENCE",
    "Set it to `1` to prefer the least stable installed Edge channel over the stable runtime.",
  );

  pub const WEBVIEW2_USER_DATA_FOLDER: TextFlag = Self::webview2(
    "WEBVIEW2_USER_DATA_FOLDER",
    "Folder for the webview's profile, instead of the application's default.",
  );

  pub const BUILD_KIND: ChoiceFlag = ChoiceFlag::new(EnvFlag {
    name: "XRF_BUILD_KIND",
    kind: EnvFlagKind::Build,
    value: EnvFlagValue::Choice(&["development", "optimized"]),
    default: "`local`",
    readers: &["xrf-build-info", "xrf-app"],
    condition: None,
    description: "Kind the build reports itself as, matched exactly. It is a label, not a Cargo profile; CI sets it.",
  });

  pub const BUILD_TIMESTAMP: TextFlag = TextFlag::new(EnvFlag {
    name: "XRF_BUILD_TIMESTAMP",
    kind: EnvFlagKind::Build,
    value: EnvFlagValue::Text,
    default: "the time the build script runs",
    readers: &["xrf-build-info"],
    condition: None,
    description: "Build time the build reports, copied as given (use RFC 3339). An empty value leaves it out.",
  });

  pub const GITHUB_SHA: TextFlag = Self::ci(
    "GITHUB_SHA",
    "the checkout's `HEAD`",
    "Commit the build reports. GitHub Actions sets it.",
  );

  pub const GITHUB_REF_NAME: TextFlag = Self::ci(
    "GITHUB_REF_NAME",
    "the checkout's branch",
    "Branch or tag the build reports. GitHub Actions sets it.",
  );

  pub const GITHUB_RUN_ID: TextFlag = Self::ci(
    "GITHUB_RUN_ID",
    "none, which leaves it out",
    "Workflow run the build reports. GitHub Actions sets it.",
  );

  pub const PACKAGE_NAME: TextFlag = TextFlag::new(EnvFlag {
    name: "PACKAGE_NAME",
    kind: EnvFlagKind::Build,
    value: EnvFlagValue::Text,
    default: "`xrf-app-windows-x64`",
    readers: &["cargo make package-app"],
    condition: None,
    description: "Name of the archive `package-app` packs the release application into, under `target/package/`.",
  });

  pub const DLSS_SDK: PathFlag = Self::sdk(
    "DLSS_SDK",
    "`.sdks/dlss`",
    "Location of NVIDIA's DLSS SDK, for builds with the `dlss` feature.",
  );

  pub const VULKAN_SDK: PathFlag = Self::sdk(
    "VULKAN_SDK",
    "`.sdks/vulkan-headers`",
    "Location of the Vulkan headers the DLSS bindings are generated against.",
  );

  pub const LIBCLANG_PATH: PathFlag = Self::sdk(
    "LIBCLANG_PATH",
    "`.sdks/libclang` on Windows; the system's elsewhere",
    "Location of the libclang that `bindgen` uses to generate the DLSS bindings.",
  );

  pub const DLSS_RUNTIME: PathFlag = PathFlag::new(EnvFlag {
    name: "XRF_DLSS_RUNTIME",
    kind: EnvFlagKind::Internal,
    value: EnvFlagValue::Path,
    default: "set by the build script",
    readers: &["xrf-dlss"],
    condition: Some("builds with the `dlss` feature"),
    description: "SDK runtime directory compiled into the binary as the last place to look for the DLSS library.",
  });

  /// Every variable, in the order the document lists them.
  pub const ALL: &'static [EnvFlag] = &[
    *Self::RENDER_BACKEND.get_flag(),
    *Self::RENDER_RAY_QUERY.get_flag(),
    *Self::APP_EXTENSIONS_DIR.get_flag(),
    *Self::RUST_LOG.get_flag(),
    *Self::WGPU_VALIDATION.get_flag(),
    *Self::WGPU_DEBUG.get_flag(),
    *Self::WGPU_GPU_BASED_VALIDATION.get_flag(),
    *Self::WGPU_VALIDATION_INDIRECT_CALL.get_flag(),
    *Self::WGPU_DISCARD_HAL_LABELS.get_flag(),
    *Self::WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER.get_flag(),
    *Self::WGPU_STRICT_WEBGPU_COMPLIANCE.get_flag(),
    *Self::WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS.get_flag(),
    *Self::WEBVIEW2_BROWSER_EXECUTABLE_FOLDER.get_flag(),
    *Self::WEBVIEW2_RELEASE_CHANNEL_PREFERENCE.get_flag(),
    *Self::WEBVIEW2_USER_DATA_FOLDER.get_flag(),
    *Self::BUILD_KIND.get_flag(),
    *Self::BUILD_TIMESTAMP.get_flag(),
    *Self::GITHUB_SHA.get_flag(),
    *Self::GITHUB_REF_NAME.get_flag(),
    *Self::GITHUB_RUN_ID.get_flag(),
    *Self::PACKAGE_NAME.get_flag(),
    *Self::DLSS_SDK.get_flag(),
    *Self::VULKAN_SDK.get_flag(),
    *Self::LIBCLANG_PATH.get_flag(),
    *Self::DLSS_RUNTIME.get_flag(),
  ];

  /// The document's sections: each kind with its title and summary.
  const SECTIONS: [(EnvFlagKind, &'static str, &'static str); 4] = [
    (
      EnvFlagKind::Runtime,
      "Runtime",
      "Read by a running binary. Set them in the environment that starts it.",
    ),
    (
      EnvFlagKind::PassThrough,
      "Passed through",
      "Read by third-party libraries. The application logs them at startup along with its own.",
    ),
    (
      EnvFlagKind::Build,
      "Build",
      "Read during the build, by a build script or CI.",
    ),
    (
      EnvFlagKind::Internal,
      "Internal",
      "Set by a build script for the compiler. Never set them by hand; they are listed so the names are not reused.",
    ),
  ];

  /// The state of every variable a running binary reads, set or not, in `ALL`'s order.
  pub fn list_readings() -> Vec<EnvFlagReading> {
    Self::ALL
      .iter()
      .filter(|flag| flag.kind.is_read_at_run())
      .map(EnvFlag::to_reading)
      .collect()
  }

  /// The variables `reader` reads at run time, for its help text.
  pub fn list_read_by(reader: &str) -> Vec<&'static EnvFlag> {
    Self::ALL
      .iter()
      .filter(|flag| flag.kind.is_read_at_run() && flag.readers.contains(&reader))
      .collect()
  }

  /// The `ENVIRONMENT.md` document for every variable.
  pub fn to_markdown() -> String {
    let mut document: String = String::from(
      "# Environment variables\n\n\
       Every environment variable the workspace reads. This file is generated from `EnvFlags::ALL` in \
       `crates/xrf-env-flags`: edit the table there, and its test rewrites this file.\n\n\
       Runtime switches accept `1`, `true`, `on` or `yes` for on and `0`, `false`, `off` or `no` for off. Runtime \
       choices accept their names in any case. An unrecognised value is logged as a warning, and the default \
       applies. Passed-through and build variables follow their own readers' rules, described with each. The \
       application logs every variable that is set at startup and lists them in Settings, About.\n",
    );

    for (kind, title, summary) in Self::SECTIONS {
      let _ = write!(document, "\n## {title}\n\n{summary}\n");

      for flag in Self::ALL.iter().filter(|flag| flag.kind == kind) {
        let _ = write!(document, "\n### `{}`\n\n{}\n\n", flag.name, flag.description);
        let _ = writeln!(document, "- Takes: {}.", flag.value.describe());
        let _ = writeln!(document, "- Default: {}.", flag.default);

        let readers: Vec<String> = flag.readers.iter().map(|reader| format!("`{reader}`")).collect();
        let _ = writeln!(document, "- Read by: {}.", readers.join(", "));

        if let Some(condition) = flag.condition {
          let _ = writeln!(document, "- Only in: {condition}.");
        }
      }
    }

    document
  }

  /// One of wgpu's instance flags, read as any switch and applied when the renderer creates its instance; `default` is
  /// what wgpu's build configuration sets.
  const fn wgpu_switch(name: &'static str, default: &'static str, description: &'static str) -> SwitchFlag {
    SwitchFlag::new(EnvFlag {
      name,
      kind: EnvFlagKind::Runtime,
      value: EnvFlagValue::Switch,
      default,
      readers: &["xrf-app"],
      condition: None,
      description,
    })
  }

  /// One of GitHub Actions' variables, which the build reports where CI sets it.
  const fn ci(name: &'static str, default: &'static str, description: &'static str) -> TextFlag {
    TextFlag::new(EnvFlag {
      name,
      kind: EnvFlagKind::Build,
      value: EnvFlagValue::Text,
      default,
      readers: &["xrf-build-info"],
      condition: None,
      description,
    })
  }

  /// One of the WebView2 runtime's variables, which the runtime reads when the window's webview starts.
  const fn webview2(name: &'static str, description: &'static str) -> TextFlag {
    TextFlag::new(EnvFlag {
      name,
      kind: EnvFlagKind::PassThrough,
      value: EnvFlagValue::Text,
      default: "the runtime's default",
      readers: &["xrf-app"],
      condition: Some("Windows"),
      description,
    })
  }

  /// A build input that `.cargo/config.toml` points at the fetched SDKs; a value set in the environment wins.
  const fn sdk(name: &'static str, default: &'static str, description: &'static str) -> PathFlag {
    PathFlag::new(EnvFlag {
      name,
      kind: EnvFlagKind::Build,
      value: EnvFlagValue::Path,
      default,
      readers: &["xrf-dlss"],
      condition: Some("builds with the `dlss` feature"),
      description,
    })
  }
}
