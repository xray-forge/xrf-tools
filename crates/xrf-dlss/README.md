# xrf-dlss

NVIDIA DLSS super resolution and ray reconstruction for the renderer's views, through NGX on wgpu's Vulkan and D3D12
backends.

[xrf-renderer](../xrf-renderer/README.md) is its one user: it starts its devices through this crate and resolves a view
with an upscaler or a ray reconstructor in place of its own upscaling.

## The `sdk` feature

Without `sdk`, the crate keeps the same types but no device has DLSS, so code using them compiles everywhere:

- `DlssInstance` starts plain devices.
- `request_device` always answers a `DlssTask` with no DLSS.
- `Dlss`, `DlssUpscaler` and `DlssRayReconstructor` cannot be made.
- `Dlss::IS_BUILT` says which build this is.

The renderer's `dlss` feature, and through it the app's, turns `sdk` on.

Building with `sdk` needs:

- NVIDIA's DLSS SDK (`DLSS_SDK`), the Vulkan SDK (`VULKAN_SDK`) and libclang. `cargo make fetch-sdks` sets them up, as
  [DEVELOPMENT.md](../../DEVELOPMENT.md) describes. On Windows with no `LIBCLANG_PATH`, the build uses the libclang
  `fetch-sdks` placed in `.sdks/libclang`.
- An x86-64 Windows or Linux target, the only ones NGX's static library ships for.

The build script binds NGX's C API with bindgen, compiles its inline helpers, links the static library, and records
where the SDK's runtime library lies.

The NGX glue is derived from dlss_wgpu (MIT OR Apache-2.0, `LICENSE-dlss_wgpu-MIT` and `LICENSE-dlss_wgpu-APACHE`).

## Start DLSS on a device

DLSS must be requested before the device exists, so the renderer makes its wgpu instance through `DlssInstance`.

```rust,ignore
let instance = DlssInstance::new(descriptor, true, Some(runtime_directory));
let adapter = pollster::block_on(instance.instance.request_adapter(&options))?;
let (device, queue, dlss) = instance.request_device(&adapter, &device_descriptor)?;

// Other start-up work here runs while NGX starts.
if let Some(dlss) = dlss.get() {
  let upscaler = dlss.create_upscaler((1920, 1080), (DlssQuality::Quality, DlssPreset::Default), true)?;
  let ngx: wgpu::CommandBuffer = upscaler.record(&mut encoder, &inputs)?;

  queue.submit([encoder.finish(), ngx]);
}
```

**`DlssInstance::new(descriptor, is_upscaled, runtime)`** makes the instance; `instance` is the plain `wgpu::Instance`.

- With `is_upscaled` and Vulkan as the only backend, it adds the instance extensions NGX needs.
- With D3D12 as the only backend (Windows only), it prepares NGX for the device.
- Any other backend set gives a plain instance, as does a Vulkan instance NGX's extensions fail on (with a warning).
- `runtime` is the directory the application ships NGX's runtime library in (`Dlss::RUNTIME_LIBRARY`,
  `nvngx_dlss.dll` on Windows, with ray reconstruction's beside it). NGX also searches the SDK the build used.

**`request_device(adapter, descriptor)`** answers the device, its queue and a `DlssTask`.

- On Vulkan it adds NGX's device extensions where the adapter has them; on D3D12 it starts NGX only on an NVIDIA
  adapter.
- NGX takes about a second to start, so it starts on its own thread (`xrf-dlss`) and the device is answered at once.
- `DlssTask::get` waits for the rest of the start and answers `Option<Dlss>`: `None`, logged, for a GPU or driver
  without DLSS. Dropping the task also waits for the start.
- Only wgpu failing to create the device is an error.

`Dlss` is cheap to clone and shares NGX between a device's views. `has_ray_reconstruction` says whether the GPU and
driver also support ray reconstruction.

## Upscale a view

Each view makes its own feature:

- `create_upscaler(output, (quality, preset), is_hdr)` makes a `DlssUpscaler` for super resolution. With `is_hdr`, the
  colour is linear light and NGX applies exposure itself.
- `create_ray_reconstructor(output, (quality, preset))` makes a `DlssRayReconstructor`, which denoises and upscales.
  Its colour must be linear light already exposed: ray reconstruction applies no exposure.

| `DlssQuality`      | Render size per side |
| ------------------ | -------------------- |
| `Native` (DLAA)    | 100%                 |
| `Quality`          | about 2/3            |
| `Balanced`         | about 58%            |
| `Performance`      | 1/2                  |
| `UltraPerformance` | 1/3                  |

The preset is a `DlssPreset` or `DlssRayPreset`. `Default` is NVIDIA's pick, which a driver update may change.

Both feature types answer:

- `get_render_range`: the smallest and largest render size they accept.
- `is_fitting`: whether they still match a view's settings, or a new one is needed.
- `record(encoder, inputs)`, each frame: writes the inputs' barriers into `encoder` and answers a command buffer of
  NGX's work. Submit it right after `encoder`'s commands and before anything recorded later.

### Inputs

`DlssInputs` and `DlssRayInputs` hold texture views at the render size:

- the colour;
- a reversed depth (one at the near plane);
- each pixel's motion back to where it was the frame before, with `motion_scale` converting it to render pixels;
- the sample jitter in render pixels, and the render size;
- the output: a storage texture at the output size;
- `is_reset`, which forgets the history on the first frame or after a cut.

Ray reconstruction also takes:

- diffuse and specular albedo;
- normals, with roughness in the fourth channel;
- hit distances of reflection and indirect rays, in metres;
- the unjittered view and projection;
- optionally, the frame before and after its transparent layers.

Neither feature takes a current-colour bias mask: NGX's `pInBiasCurrentColorMask` serves super resolution's preset F
alone, the transformer presets ignore it (DLSS Programming Guide, "Biasing the Current Frame"), and ray reconstruction's
models E and F drew water rippling in rain alike with a mask of 0.8 over it and with none.

NGX's own messages are logged under the `xrf_dlss::ngx` target while its debug level is on.

## Checks

Run `cargo test --locked -p xrf-dlss`; add `--features sdk` to build the NGX glue.

Sources: the [instance](src/dlss_instance.rs), [the start's task](src/dlss_task.rs), [`Dlss`](src/dlss.rs), the
[upscaler](src/upscaler/dlss_upscaler.rs), the [ray reconstructor](src/reconstruction/dlss_ray_reconstructor.rs), and
the [build script](build.rs). The NGX glue under `src/ngx` is split into starting NGX on a device (`sdk`), the features
it makes (`feature`), and the graphics APIs it runs on (`backend`).
