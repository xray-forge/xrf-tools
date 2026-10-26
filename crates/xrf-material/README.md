# xrf-material

What the X-Ray renderer builds for a surface, answered the way the engine would. A surface declares two things, and
each is resolved separately, because neither is derivable from the other and neither is in the mesh:

- From a texture's `.thm` descriptor, and a loaded level's own over it: the bump pair, the detail association, the
  lighting model, and the outcome of resolving them, the way `CTextureDescrMngr` and `CRender::texture_load` read them.
- From a shader name, a renderer shader script or `shaders.xr`: whether alpha is read at all and how, the way the
  blender's own `Compile` decides it.

The renderer, the desktop app's level, model and texture views, and the texture verifier ask it what to draw. Binary
THM parsing lives in [xrf-thm](../xrf-thm/README.md), and the shader library and scripts in
[xrf-shaders](../xrf-shaders/README.md).

## Describe a material and surface

```rust,no_run
use xrf_material::{XrayMaterialResolver, XraySurfaceResolver, XrayTextureScope};
use xrf_vfs::{XrayMountMode, XraySearchPlan};

fn main() -> xrf_error::XrfResult {
  let search = XraySearchPlan::new()
    .with_root_mode("gamedata", "gamedata", XrayMountMode::Directory)?
    .open()?;
  let material = XrayMaterialResolver::describe_texture(&search, &XrayTextureScope::shared(), "wpn\\wpn_ak74");

  println!("Bump outcome: {:?}", material.outcome);

  let surfaces = XraySurfaceResolver::open(&search, XrayTextureScope::shared());
  let surface = surfaces.describe("models\\model", &["wpn\\wpn_ak74".to_owned()]);

  println!("Declaration: {:?}; draw: {:?}", surface.declaration, surface.draw);

  Ok(())
}
```

The texture reference and shader name answer different questions. A mesh's diffuse texture name does not determine its
alpha draw mode; that comes from its shader. The caller mounts the sources and decides which roots the `XraySearch`
covers and in what order.

## Choose a resolver

- `XrayMaterialResolver::describe_texture(search, scope, reference)` finds a texture's `.thm` and resolves its bump pair
  and detail association. `describe_descriptor` starts from a THM asset already located by a scan, outside any level,
  and `describe_read_descriptor` from a read the caller already made; `read_descriptor` reads one through the search's
  cache.
- Open `XraySurfaceResolver` once with a search and a scope, then call `describe(shader_name, textures)` for each
  surface. It reads the shader library once and reports both the authored declaration and the resulting draw mode.
  Reuse one resolver across every surface using the same library; open a fresh one when the library changes.
- `XrayTextureScope` says where the renderer looks: `shared()` outside a level, `of_level(directory)` inside one. In a
  level, the level's own `.thm` is read over the shared one as `LoadTHM("$level$")` does: it replaces the bump and the
  material, and the detail only where it names a live one, and one the type gate skips changes nothing. Textures and
  bump inputs resolve beside the level first, save that a `_bump` name the shared tree lacks is the engine's dummy;
  `resolve_texture` answers that order for any other texture a level binds, and `locate_texture` the logical path it
  lands on, a reference the search refuses reading as absent.
- `XrayReducedTextures` answers whether a texture loads a step coarser, from `system.ltx`'s `reduce_lod_texture_list`.

## Texture dependencies

`XrayMaterialDescriptor` carries the located THM, its declaration, bump outcome, optional bound bump pair, optional
detail association, and `material`, the lighting model (`1.0` where no descriptor is read).
`XrayMaterialDeclaration` keeps apart the ways a texture ends up flat: no descriptor, an unreadable one, one whose
texture type `LoadTHM` skips whole, no bump chunk, a disabled mode, or an empty name. Only `Declared` binds a pair. A
missing descriptor does not mean the diffuse texture itself is absent.

`XrayBumpOutcome` is the worse of the pair's two inputs: `Flat`, `Bumped`, `Dummy` (the bump shader runs against the
engine's flat dummy because a `_bump` file is absent), or `Missing` (an input fell to `ed\ed_not_existing_texture` or to
nothing). Inspect both the declaration and the outcome to distinguish what was requested from what was found or
substituted. `describe_descriptor` is useful for sweeping THMs even when no diffuse texture with the same name exists.

## Alpha behavior

Surfaces are described for OpenXRay's DX11 renderer, the one it runs by default. `XraySurfaceResolver::describe`
looks for its shader script, `shaders\r3\<name>.s` with `\` in the name read as `_`, before the library, because the
engine uses a script in place of the library entry when one exists. Scripts in both the `sampler():texture()` and the
`dx10texture` dialects are read, and a script's `normal_hq` pass is the one compiled where the base texture associates
a detail. A scripted surface reports `XraySurfaceDeclaration::Scripted` and the texture each of its samplers binds.
Otherwise the blender's class decides: `Described` with its alpha switch, reference and strict sorting, or
`Undeclared`, `NoLibrary`, `Unreadable`, `Undefined` or `Unmodelled` where nothing could be said. A knob a library
omits reads as the value its class starts from: `Alpha ref` 32, or 200 for the level cut-out classes, and
`Texture clamp` on.

`XraySurfaceDescriptor` keeps the declaration separate from `XraySurfaceDraw`: `Opaque`, `AlphaTested`, `Blended`,
`Added`, `Multiplied`, `Invisible`, or `Water`. A model asking for strict sorting, or for alpha below a reference of 16,
blends forward rather than cutting out. A surface nothing could be read for still draws `Opaque`, as DX11 draws a
shader it cannot resolve with `stub_default.s`; do not treat a drawable fallback as proof that the authored shader was
found and understood. The descriptor also carries the detail texture, terrain layers, bump pair, environment cube, and
the clamp, distortion, emission, shadow and object-LOD flags. The bump pair and the detail are bound only on a deferred
pass: a blended surface's forward pass samples its base alone.

## Limitations, features, and checks

`find_textures_ltx` locates a possible `textures\textures.ltx` override file but does not interpret it. The crate
reports descriptors and resolutions; it does not decode diffuse pixels, compile shaders, or render a scene. The
resolver methods express missing and unreadable declarations in result values rather than errors, so callers must
inspect those values even though no `Result` is returned.

The optional `fixtures` feature exposes the `fixtures` test builders; `typescript-bindings` derives frontend type
metadata.

Run `cargo test --locked -p xrf-material` from the repository root.

See the [texture resolver](src/resolve/xray_material_resolver.rs), the
[surface resolver](src/resolve/xray_surface_resolver.rs), the [texture scope](src/resolve/xray_texture_scope.rs), and
the [draw modes](src/data/xray_surface_draw.rs).
