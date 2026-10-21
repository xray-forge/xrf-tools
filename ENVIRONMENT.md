# Environment variables

Every environment variable the workspace reads. This file is generated from `EnvFlags::ALL` in `crates/xrf-env-flags`: edit the table there, and its test rewrites this file.

Runtime switches accept `1`, `true`, `on` or `yes` for on and `0`, `false`, `off` or `no` for off. Runtime choices accept their names in any case. An unrecognised value is logged as a warning, and the default applies. Passed-through and build variables follow their own readers' rules, described with each. The application logs every variable that is set at startup and lists them in Settings, About.

## Runtime

Read by a running binary. Set them in the environment that starts it.

### `XRF_RENDER_BACKEND`

Starts the renderer on this graphics API instead of the one chosen in the settings. If it cannot start, the renderer falls back to the other APIs of the platform: D3D12 and Vulkan on Windows, Metal on macOS (experimental).

- Takes: `d3d12`, `vulkan` or `metal`.
- Default: the API chosen in the settings.
- Read by: `xrf-app`.

### `XRF_RENDER_RAY_QUERY`

Set it off to stop the renderer using hardware ray queries. Traced effects then use their untraced versions.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: on when the GPU supports ray queries.
- Read by: `xrf-app`.

### `XRF_APP_EXTENSIONS_DIR`

Directory to load unpacked browser extensions into the webview from. Each entry must be a folder with a `manifest.json`.

- Takes: a path.
- Default: `bin/xrf-app/extensions/unpacked` in the checkout.
- Read by: `xrf-app`.
- Only in: debug builds on Windows.

### `WGPU_VALIDATION`

Turns wgpu's API validation on or off.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: on in debug builds, off otherwise.
- Read by: `xrf-app`.

### `WGPU_DEBUG`

Turns wgpu's debug labels and markers on GPU objects on or off, for graphics debuggers.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: on in debug builds, off otherwise.
- Read by: `xrf-app`.

### `WGPU_GPU_BASED_VALIDATION`

Turns on the driver's GPU-based validation. It is much slower and needs `WGPU_VALIDATION` on.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: off.
- Read by: `xrf-app`.

### `WGPU_VALIDATION_INDIRECT_CALL`

Turns wgpu's validation of indirect draw and dispatch arguments on or off.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: on.
- Read by: `xrf-app`.

### `WGPU_DISCARD_HAL_LABELS`

When on, wgpu does not pass labels down to the graphics API's objects.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: off.
- Read by: `xrf-app`.

### `WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER`

When on, wgpu also offers adapters it does not consider compliant.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: off.
- Read by: `xrf-app`.

### `WGPU_STRICT_WEBGPU_COMPLIANCE`

When on, wgpu enforces WebGPU's limits strictly.

- Takes: `1`, `true`, `on` or `yes` for on; `0`, `false`, `off` or `no` for off.
- Default: off.
- Read by: `xrf-app`.

## Passed through

Read by third-party libraries. The application logs them at startup along with its own.

### `RUST_LOG`

Log filter in `env_logger` syntax, for example `info,wgpu_hal=warn`.

- Takes: text.
- Default: `info` for xrf-app (`debug` in a debug build), `error` for xrf-cli (`warn` in a debug build).
- Read by: `xrf-app`, `xrf-cli`.

### `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`

Extra Chromium switches for the webview, for example `--auto-open-devtools-for-tabs`.

- Takes: text.
- Default: the runtime's default.
- Read by: `xrf-app`.
- Only in: Windows.

### `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`

Folder of a fixed-version WebView2 runtime to use instead of the installed one.

- Takes: text.
- Default: the runtime's default.
- Read by: `xrf-app`.
- Only in: Windows.

### `WEBVIEW2_RELEASE_CHANNEL_PREFERENCE`

Set it to `1` to prefer the least stable installed Edge channel over the stable runtime.

- Takes: text.
- Default: the runtime's default.
- Read by: `xrf-app`.
- Only in: Windows.

### `WEBVIEW2_USER_DATA_FOLDER`

Folder for the webview's profile, instead of the application's default.

- Takes: text.
- Default: the runtime's default.
- Read by: `xrf-app`.
- Only in: Windows.

## Build

Read during the build, by a build script or CI.

### `XRF_BUILD_KIND`

Kind the build reports itself as, matched exactly. It is a label, not a Cargo profile; CI sets it.

- Takes: `development` or `optimized`.
- Default: `local`.
- Read by: `xrf-build-info`, `xrf-app`.

### `XRF_BUILD_TIMESTAMP`

Build time the build reports, copied as given (use RFC 3339). An empty value leaves it out.

- Takes: text.
- Default: the time the build script runs.
- Read by: `xrf-build-info`.

### `GITHUB_SHA`

Commit the build reports. GitHub Actions sets it.

- Takes: text.
- Default: the checkout's `HEAD`.
- Read by: `xrf-build-info`.

### `GITHUB_REF_NAME`

Branch or tag the build reports. GitHub Actions sets it.

- Takes: text.
- Default: the checkout's branch.
- Read by: `xrf-build-info`.

### `GITHUB_RUN_ID`

Workflow run the build reports. GitHub Actions sets it.

- Takes: text.
- Default: none, which leaves it out.
- Read by: `xrf-build-info`.

### `PACKAGE_NAME`

Name of the archive `package-app` packs the release application into, under `target/package/`.

- Takes: text.
- Default: `xrf-app-windows-x64`.
- Read by: `cargo make package-app`.

### `DLSS_SDK`

Location of NVIDIA's DLSS SDK, for builds with the `dlss` feature.

- Takes: a path.
- Default: `.sdks/dlss`.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.

### `VULKAN_SDK`

Location of the Vulkan headers the DLSS bindings are generated against.

- Takes: a path.
- Default: `.sdks/vulkan-headers`.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.

### `LIBCLANG_PATH`

Location of the libclang that `bindgen` uses to generate the DLSS bindings.

- Takes: a path.
- Default: `.sdks/libclang` on Windows; the system's elsewhere.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.

## Internal

Set by a build script for the compiler. Never set them by hand; they are listed so the names are not reused.

### `XRF_DLSS_RUNTIME`

SDK runtime directory compiled into the binary as the last place to look for the DLSS library.

- Takes: a path.
- Default: set by the build script.
- Read by: `xrf-dlss`.
- Only in: builds with the `dlss` feature.
