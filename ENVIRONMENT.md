# Environment variables

Every environment variable the workspace reads or honours. Generated from `crates/xrf-env-flags` (`EnvFlags::ALL`); edit the table there, and its test rewrites this file.

A runtime switch takes `1`, `true`, `on` or `yes` for on and `0`, `false`, `off` or `no` for off, and a runtime choice takes its names in any case; one set to what it does not take is warned of in the log, and its default applies. Passed-through and build variables are read as their readers read them, as each says. The application logs every variable set at startup, and lists them in Settings, About.

## Runtime

Read by a running binary: set them in the environment that starts it.

### `XRF_RENDER_BACKEND`

The graphics API the renderer starts on, over the settings' choice; one that cannot start falls back to the other.

- Takes: `d3d12` or `vulkan`.
- Unset: the settings' choice.
- Read by: `xrf-app`.

### `XRF_RENDER_RAY_QUERY`

Off leaves hardware ray queries unused, so the traced effects fall back to their untraced forms.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Unset: on where the GPU has ray queries.
- Read by: `xrf-app`.

### `XRF_APP_EXTENSIONS_DIR`

The directory unpacked browser extensions are loaded into the webview from, every entry a folder with a `manifest.json`.

- Takes: a path.
- Unset: `bin/xrf-app/extensions/unpacked` in the checkout.
- Read by: `xrf-app`.
- Only in: debug builds on Windows.

## Passed through

Other libraries' own, which they read themselves; the application reports them with its own at startup.

### `RUST_LOG`

The log's filter, in `env_logger`'s syntax, such as `info,wgpu_hal=warn`.

- Takes: text.
- Unset: `info` for xrf-app (`debug` in a debug build), `error` for xrf-cli (`warn` in a debug build).
- Read by: `xrf-app`, `xrf-cli`.

### `WGPU_VALIDATION`

wgpu's validation of every call, as debug builds have it.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WGPU_DEBUG`

wgpu's debug labels and markers on GPU objects, for graphics debuggers.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WGPU_GPU_BASED_VALIDATION`

The driver's GPU-based validation, far slower; with `WGPU_VALIDATION` on.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WGPU_VALIDATION_INDIRECT_CALL`

wgpu's validation of indirect draw and dispatch arguments.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WGPU_DISCARD_HAL_LABELS`

Labels left off the graphics API's own objects.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER`

Adapters wgpu does not consider compliant offered anyway.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WGPU_STRICT_WEBGPU_COMPLIANCE`

WebGPU's limits enforced strictly.

- Takes: `0` for off, any other value for on (`false` too).
- Unset: as the build has it: wgpu turns validation and debug labels on in a debug build.
- Read by: `xrf-app`.

### `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`

Chromium switches the webview starts with, such as `--auto-open-devtools-for-tabs`.

- Takes: text.
- Unset: the runtime's own.
- Read by: `xrf-app`.
- Only in: Windows.

### `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`

A fixed-version WebView2 runtime to use instead of the installed one.

- Takes: text.
- Unset: the runtime's own.
- Read by: `xrf-app`.
- Only in: Windows.

### `WEBVIEW2_RELEASE_CHANNEL_PREFERENCE`

`1` looks for the least stable installed Edge channel first, rather than the stable runtime.

- Takes: text.
- Unset: the runtime's own.
- Read by: `xrf-app`.
- Only in: Windows.

### `WEBVIEW2_USER_DATA_FOLDER`

The webview's profile directory instead of the application's.

- Takes: text.
- Unset: the runtime's own.
- Read by: `xrf-app`.
- Only in: Windows.

## Build

Read while the workspace builds, by a build script or CI.

### `XRF_BUILD_KIND`

What the build reports itself as, matched exactly; a label, not a profile: CI sets it.

- Takes: `development` or `optimized`.
- Unset: `local`.
- Read by: `xrf-build-info`.

### `XRF_BUILD_TIMESTAMP`

When the build reports it was made, copied as given (RFC 3339); empty leaves it out.

- Takes: text.
- Unset: the time the build script runs.
- Read by: `xrf-build-info`.

### `DLSS_SDK`

NVIDIA's DLSS SDK, for a build with the `dlss` feature.

- Takes: a path.
- Unset: `.sdks/dlss`.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.

### `VULKAN_SDK`

The Vulkan headers DLSS's bindings are generated against.

- Takes: a path.
- Unset: `.sdks/vulkan-headers`.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.

### `LIBCLANG_PATH`

The libclang `bindgen` generates DLSS's bindings with.

- Takes: a path.
- Unset: `.sdks/libclang` on Windows, the system's elsewhere.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.

## Internal

Set by a build script for the compiler, never by hand; listed so the name is not reused.

### `XRF_DLSS_RUNTIME`

The SDK's runtime directory, compiled in as the last place the DLSS library is looked for.

- Takes: a path.
- Unset: set by the build script.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.
