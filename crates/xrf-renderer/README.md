# xrf-renderer

The native renderer: one GPU and one render thread drawing viewports into the application's windows. Every viewer of
the desktop app draws through it, under a transparent webview: the level viewer, the mesh viewers and the texture body.
It is built on wgpu and on [xrf-renderer-core](../xrf-renderer-core/README.md)'s frame graph; what moves and plays in a
level comes from [xrf-world](../xrf-world/README.md), and DLSS from [xrf-dlss](../xrf-dlss/README.md).

## What the host provides

The renderer takes what it draws in memory and never opens a file. It knows the application hosting it only through
these traits, which the app implements:

| Trait               | What the host gives                                                                        |
| ------------------- | ------------------------------------------------------------------------------------------ |
| `RenderLevelSource` | An open level as a scene draws it: its sector count and shader table's surfaces            |
| `RenderAssetSource` | Texture bytes by reference, within a scope two sources share when they read the same files |
| `RenderWindowHost`  | A native window to draw into: its surface target, client size, and whether it is hidden    |
| `RenderEventSink`   | Where one viewport's `RenderViewportEvent`s go: in the app, the channel its page attached  |
| `RenderWorld`       | What moves and plays, run on the render thread before each frame (`xrf-world`'s `World`)   |
| `RenderCacheStore`  | Bytes kept between runs: the driver's compiled pipelines                                   |
| `RenderBundle`      | The files the renderer ships beside its code, under `assets/` (`RENDER_BUNDLE_SOURCE`)     |
| `RenderWorkers`     | The application's own thread pool, which every loader reads and decodes on                 |

`RenderWindowHost` is an `unsafe trait`: the surface target it answers must name a live window, or a layer drawn in
one, until every viewport attached with it is detached and the render thread has handled that detach
(`RenderQueue::wait_handled` answering anything but a timeout). The renderer presents into it until then.
What a scene takes in (packed sectors, lights, spawn, grass, particles, the level's weather) the host reads through
`xrf-world`'s `WorldLevelSource`, which extends `RenderLevelSource`, and hands over each frame.

## The render thread

`RenderQueue::new(workers, bundle, world)` is the way in: cheap to make, and touching no GPU until a viewport is
attached. Every `RenderCommand` sent is a message to the render thread, the only one touching the GPU. The thread is
started by the first `Attach` and stopped a few seconds after the last detach, so a tool that never draws never starts
a GPU; a command other than an attach with no thread running does nothing, but `Settings` are kept for the next
thread. A clone reaches the same thread.

- `Attach` gives a viewport its id, window host and event sink; `Layout`, `Options`, `Overlays` and `Selection` steer
  it; `Pick`, `Describe*`, `Capture`, `Screenshot`, `CaptureMap` and `CaptureCube` answer on a channel. `Pick`,
  `Capture` and `Screenshot` answer after the viewport's next frame and never for a viewport the renderer does not
  draw, so wait on them with a timeout.
- `wait_handled(wait)` waits until every command sent before is handled; `stop(wait)` detaches every viewport and waits
  until the GPU is let go.
- `with_cache_store` keeps the driver's compiled pipelines between runs.

The backend is a `RenderBackend`: D3D12 and Vulkan where the platform builds them, tried in that order when the one
asked for cannot start, and Metal alone on Apple platforms (experimental: no CI job builds or tests it).
`XRF_RENDER_BACKEND` overrides the settings' choice; [ENVIRONMENT.md](../../ENVIRONMENT.md) lists it and wgpu's
switches, which the renderer reads as its own.

## The contract

The `Render*` types under `src/contract/` cross IPC to the page: settings, view options, reports and events. Values the
page bounds or defaults by are marked with `#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]` where they
are declared ([xrf-ipc](../xrf-ipc/README.md)). After changing either, regenerate the frontend mirrors with
`cargo make generate-typescript`.

## Shaders

The renderer's WGSL lives in `shaders/`. A module is named by its path under `shaders/` without the extension, and
`#import` inlines another; an import needing a feature the device lacks is composed as its fallback (the scene trace
without ray queries). Every module is embedded from the list in `src/shader/shader_library.rs`, so a new file must be
added there. A debug build also reads them from the checkout and watches them, so an edited module is drawn with on
the next reload.

`shaders/generated/` is written from Rust: the structs shaders share with Rust (`ShaderStruct`), and each module's
bindings gathered from the parameters of every pass drawing with it (`PassParameters`). The test
`every_generated_module_is_up_to_date` rewrites a stale file and fails once; commit the rewritten file.

## Features

| Feature               | Effect                                                                                    |
| --------------------- | ----------------------------------------------------------------------------------------- |
| `dlss`                | Upscales with NVIDIA DLSS on Vulkan and D3D12, built against its SDK (`xrf-dlss/sdk`)     |
| `tracy`               | Zones, GPU zones and frame marks for a Tracy capture (`xrf-renderer-core/tracy`)          |
| `typescript-bindings` | Derives the contract's Specta types and registers its constants for the generated mirrors |

The renderer adapts third-party work (AMD FSR among it); [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) is the one
place their credits live.

## Checks

Run `cargo test --locked -p xrf-renderer`. Tests that draw open a device through `src/tests/test_context.rs`, skipping
with a message on a machine with none, or failing where `XRF_TEST_REQUIRE_GPU` is set. Sources: the
[host traits](src/host/), the [queue](src/thread/render_queue.rs) and [commands](src/thread/render_command.rs), the
[contract](src/contract/), and the [shader library](src/shader/shader_library.rs).
