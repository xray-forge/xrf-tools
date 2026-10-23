# xrf-dlss

NVIDIA DLSS super resolution and ray reconstruction for the renderer's views, through NGX on wgpu's Vulkan and D3D12
backends. [xrf-renderer](../xrf-renderer/README.md) is its one user: it starts its devices through this crate and
resolves a view with an upscaler or a ray reconstructor in place of its own upscaling.

## The `sdk` feature

Built without the `sdk` feature, the crate offers the same types and no device has DLSS: `DlssInstance` starts plain
devices, `request_device` always answers `None` for DLSS, and `Dlss`, `DlssUpscaler` and `DlssRayReconstructor` cannot
be made, so code using them compiles everywhere. `Dlss::IS_BUILT` says which build this is. The renderer's `dlss`
feature, and through it the app's, turns `sdk` on.

Building with `sdk` needs NVIDIA's DLSS SDK (`DLSS_SDK`), the Vulkan SDK (`VULKAN_SDK`) and libclang, as
[DEVELOPMENT.md](../../DEVELOPMENT.md) sets them up (`cargo make fetch-sdks`). The build script binds NGX's C API with
bindgen, compiles its inline helpers, links its static library (Windows and Linux on x86-64 only) and records where the
SDK's runtime library lies. On a Windows host with no `LIBCLANG_PATH`, it uses the libclang `fetch-sdks` placed in
`.sdks/libclang`.

The NGX glue is derived from dlss_wgpu (MIT OR Apache-2.0, `LICENSE-dlss_wgpu-MIT` and `LICENSE-dlss_wgpu-APACHE`).

## Starting DLSS on a device

DLSS has to be asked for before the device exists, so the renderer makes its wgpu instance through `DlssInstance`:

- `DlssInstance::new(descriptor, is_upscaled, runtime)` makes the instance (`instance` is the plain `wgpu::Instance`).
  With `is_upscaled` and Vulkan alone as the backend, it adds the instance extensions NGX needs; with D3D12 alone, it
  prepares NGX for the device (Windows only). Any other backend set gives a plain instance, and so does a Vulkan
  instance NGX's extensions fail on, with a warning logged.
- `request_device(adapter, descriptor)` answers the device, its queue and `Option<Dlss>`. On Vulkan it opens the device
  with NGX's device extensions where the adapter has them; on D3D12 it starts NGX only on an NVIDIA adapter. A GPU or
  driver without DLSS answers `None`, logged; only wgpu's own failure to start the device is an error.
- `runtime` is the directory an application ships NGX's runtime library in (`Dlss::RUNTIME_LIBRARY`, `nvngx_dlss.dll`
  on Windows, with ray reconstruction's beside it). NGX also searches the SDK the build was made against.

`Dlss` is cheap to clone and shares NGX between the views of a device. `has_ray_reconstruction` says whether the GPU and
driver also reconstruct rays.

## Upscaling a view

Each view makes its own feature at an output size, a `DlssQuality` (`Native` is DLAA; `Quality`, `Balanced`,
`Performance` and `UltraPerformance` draw at about 2/3, 58%, 1/2 and 1/3 of each side) and a preset (`DlssPreset` or
`DlssRayPreset`; `Default` is NVIDIA's pick, which a driver update may change):

- `create_upscaler(output, (quality, preset), is_hdr)` makes a `DlssUpscaler` for super resolution. With `is_hdr` the
  colour is linear light and NGX exposes it itself.
- `create_ray_reconstructor(output, (quality, preset))` makes a `DlssRayReconstructor`, which denoises and upscales.
  Its colour is linear light already exposed: ray reconstruction exposes nothing itself.

Both answer `get_render_range`, the smallest and largest render size they accept, and `is_fitting`, whether they still
match a view's settings or a new one is needed. Each frame, `record(encoder, inputs)` writes the inputs' barriers into
`encoder` and answers a command buffer of NGX's work, which the caller submits right after `encoder`'s commands and
before anything recorded later.

The inputs (`DlssInputs`, `DlssRayInputs`) are texture views at the render size: the colour, a reversed depth (one at
the near plane), each pixel's motion back to where it stood the frame before (with `motion_scale` turning it into
render pixels), the sample jitter in render pixels, the render size, and the output, a storage texture at the output
size. `is_reset` forgets the history, on the first frame or after a cut. An optional `bias_current_color` mask weighs
pixels toward this frame where something drawn over the surfaces changes them. Ray reconstruction also takes the
diffuse and specular albedo, normals with roughness in the fourth channel, the hit distances of reflection and indirect
rays in metres, the unjittered view and projection, and optionally the frame before and after its transparent layers.

```rust,ignore
let instance = DlssInstance::new(descriptor, true, Some(runtime_directory));
let adapter = pollster::block_on(instance.instance.request_adapter(&options))?;
let (device, queue, dlss) = instance.request_device(&adapter, &device_descriptor)?;

if let Some(dlss) = &dlss {
  let upscaler = dlss.create_upscaler((1920, 1080), (DlssQuality::Quality, DlssPreset::Default), true)?;
  let ngx: wgpu::CommandBuffer = upscaler.record(&mut encoder, &inputs)?;

  queue.submit([encoder.finish(), ngx]);
}
```

NGX's own messages go to the log under the `xrf_dlss::ngx` target while its debug level is on.

## Checks

Run `cargo test --locked -p xrf-dlss`, adding `--features sdk` to build the NGX glue. Sources: the
[instance](src/dlss_instance.rs), [`Dlss`](src/dlss.rs), the [upscaler](src/dlss_upscaler.rs), the
[ray reconstructor](src/dlss_ray_reconstructor.rs), and the [build script](build.rs).
