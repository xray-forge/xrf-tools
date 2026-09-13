# xrf-material

Describes the texture dependencies and alpha behavior an X-Ray surface gets from THM descriptors and `shaders.xr`.

## Describe a material and surface

```rust,no_run
use xrf_material::{XrayMaterialResolver, XraySurfaceResolver};
use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayVfs};

fn main() -> xrf_error::XrfResult {
  let vfs = XrayVfs::open(XrayMountMode::Directory, "gamedata")?;
  let probe = vfs.probe().with_step("gamedata", XrayLookupScope::all());
  let material = XrayMaterialResolver::describe_texture(&probe, "wpn\\wpn_ak74");

  println!("Bump outcome: {:?}", material.outcome);

  let surfaces = XraySurfaceResolver::open(&probe);
  let surface = surfaces.describe("models\\model");

  println!("Declaration: {:?}; draw: {:?}", surface.declaration, surface.draw);

  Ok(())
}
```

The texture reference and shader name answer different questions. A mesh's diffuse texture name does not determine
its alpha draw mode; that comes from its shader declaration.

## Choose a resolver

- `XrayMaterialResolver::describe_texture(probe, reference)` finds a texture's `.thm` and resolves its bump pair and
  detail association. `describe_descriptor` starts from a THM asset already located by a scan.
- Open `XraySurfaceResolver` once with a probe, then call `describe(shader_name)` for each surface. It reads the shader
  library once and reports both the authored declaration and the resulting draw mode.

Reuse one `XraySurfaceResolver` across all surfaces using the same library. A texture descriptor is per texture, while
`shaders.xr` contains many surface declarations. Create a fresh resolver when the underlying library changes.

## Texture dependencies

`XrayMaterialDescriptor` carries the located THM, its declaration, bump outcome, optional bound bump pair, and optional
detail association. A missing descriptor is explicit; it does not mean the diffuse texture itself is absent.

Bump resolution uses the probe's ordered sources and the engine fallback rules. Inspect both the declaration and the
outcome to distinguish what was requested from what was found or substituted. `describe_descriptor` is useful for
sweeping THMs even when no diffuse texture with the same name exists.

## Alpha behavior

The caller mounts the sources and supplies an `XrayProbe`. Missing, unreadable, undefined, and unmodelled declarations
remain explicit in the result. Surface alpha rules model the deferred renderer, R2 and above.

`XraySurfaceDescriptor` keeps the declaration separate from `XraySurfaceDraw`: opaque, alpha-tested, or blended.
An absent or unmodelled declaration can still produce the opaque fallback. Do not treat a drawable fallback as proof
that the authored shader was found and understood.

## Limitations, features, and checks

`find_textures_ltx` locates a possible override file but does not interpret it. The optional `fixtures` feature exposes
test builders; `typescript-bindings` enables frontend type metadata.

The crate reports descriptors and resolutions; it does not decode diffuse pixels, compile shaders, or render a scene.
The resolver methods express missing and unreadable declarations in result values, so callers must inspect those values
even when no outer `Result` is returned.

Run `cargo test --locked -p xrf-material` from the repository root. Binary THM and shader-library parsing is provided by
[xrf-db](../xrf-db/README.md).

See the [texture resolver](src/resolve/xray_material_resolver.rs) and
[surface resolver](src/resolve/xray_surface_resolver.rs).
