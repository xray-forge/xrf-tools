# xrf-material

Answers what the X-Ray renderer builds for a surface, the way the engine would decide it.

A surface declares two independent things, and neither is stored in the mesh:

| Source                                     | Decides                                          | Engine                                       |
| ------------------------------------------ | ------------------------------------------------ | -------------------------------------------- |
| The texture's `.thm` descriptor            | Bump pair, detail texture, lighting model        | `CTextureDescrMngr`, `CRender::texture_load` |
| The shader name (a script or `shaders.xr`) | Whether and how alpha is read, and the draw mode | The blender's own `Compile`                  |

A mesh's diffuse texture name does not determine its alpha mode; its shader does.

The renderer, the desktop app's level, model and texture views, and the texture verifier use it. Binary THM parsing is
in [xrf-thm](../xrf-thm/README.md); the shader library and scripts are in [xrf-shaders](../xrf-shaders/README.md).

## Describe a material and a surface

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

The caller mounts the sources and decides which roots the `XraySearch` covers, in what order.

## Choose a resolver

**Textures.** `XrayMaterialResolver`:

- `describe_texture(search, scope, reference)` finds a texture's `.thm` and resolves its bump pair and detail.
- `describe_descriptor` starts from a THM a scan already located, outside any level. It works even when no diffuse
  texture of that name exists, which suits sweeping all THMs.
- `describe_read_descriptor` starts from a read the caller already made; `read_descriptor` reads one through the
  search's cache.

**Surfaces.** Open an `XraySurfaceResolver` once with a search and a scope, then call `describe(shader_name, textures)`
per surface. It reads the shader library once; reuse it for every surface, and open a new one when the library changes.

**Scope.** `XrayTextureScope` says where the renderer looks: `shared()` outside a level, `of_level(directory)` inside
one. In a level:

- The level's own `.thm` is read over the shared one, as `LoadTHM("$level$")` does. It replaces the bump and the
  material, and the detail only where it names a live one. A level THM the type gate skips changes nothing.
- Textures and bump inputs resolve beside the level first. A `_bump` name the shared tree lacks is the engine's dummy.
- `resolve_texture` applies that order to any other texture a level binds; `locate_texture` gives the logical path it
  lands on. A reference the search refuses reads as absent.

**Reduced textures.** `XrayReducedTextures` says whether a texture loads one step coarser, from `system.ltx`'s
`reduce_lod_texture_list`.

## Texture dependencies

`XrayMaterialDescriptor` carries the located THM, its declaration, the bump outcome, the bound bump pair and detail
texture if any, and `material`, the lighting model (`1.0` when no descriptor is read).

`XrayMaterialDeclaration` tells apart the ways a texture ends up flat:

- no descriptor, or an unreadable one;
- a texture type `LoadTHM` skips entirely;
- no bump chunk, a disabled mode, or an empty name.

Only `Declared` binds a pair. A missing descriptor does not mean the diffuse texture is missing.

`XrayBumpOutcome` is the worse of the pair's two inputs:

| Outcome   | Meaning                                                                           |
| --------- | --------------------------------------------------------------------------------- |
| `Flat`    | No bump                                                                           |
| `Bumped`  | Both inputs found                                                                 |
| `Dummy`   | A `_bump` file is absent, so the bump shader runs against the engine's flat dummy |
| `Missing` | An input fell back to `ed\ed_not_existing_texture`, or to nothing                 |

Check both the declaration (what was asked for) and the outcome (what was found or substituted).

## Alpha and draw mode

Surfaces are described for OpenXRay's DX11 renderer, its default.

**Scripts first.** `XraySurfaceResolver::describe` looks for a shader script, `shaders\r3\<name>.s` (with `\` in the
name read as `_`), before the library, because the engine uses a script instead of the library entry when one exists.

- Scripts in both the `sampler():texture()` and `dx10texture` dialects are read.
- The `normal_hq` pass is the one compiled when the base texture has a detail.
- A scripted surface reports `XraySurfaceDeclaration::Scripted` and the texture each sampler binds.

**Otherwise the blender class decides**, reporting one of:

- `Described`, with its alpha switch, reference and strict sorting;
- `Undeclared`, `NoLibrary`, `Unreadable`, `Undefined` or `Unmodelled`, when nothing could be said.

A setting the library omits reads as its class's default: `Alpha ref` 32 (200 for the level cut-out classes), and
`Texture clamp` on.

**Draw mode.** `XraySurfaceDescriptor` keeps the declaration separate from `XraySurfaceDraw`: `Opaque`, `AlphaTested`,
`Blended`, `Added`, `Multiplied`, `Invisible` or `Water`.

- A model asking for strict sorting, or for alpha below a reference of 16, blends rather than cutting out.
- A surface nothing could be read for still draws `Opaque`, as DX11 draws an unresolved shader with `stub_default.s`.
  A drawable fallback does not prove the authored shader was found.
- The descriptor also carries the detail texture, terrain layers, bump pair, environment cube, and the clamp,
  distortion, emission, shadow and object-LOD flags.
- The bump pair and detail bind only on a deferred pass; a blended surface's forward pass samples its base texture
  alone.

## Limits

- Missing and unreadable declarations are result values, not errors. No `Result` is returned, so inspect the values.
- `find_textures_ltx` locates a possible `textures\textures.ltx` override but does not interpret it.
- The crate does not decode pixels, compile shaders, or render.

## Features

- `fixtures` exposes the `fixtures` test builders.
- `typescript-bindings` derives frontend type metadata.

## Checks

Run `cargo test --locked -p xrf-material` from the repository root.

The sources follow the two halves: `src/material/` reads descriptors, `src/surface/` reads shaders (each blender class's
rules in `src/surface/rule/`), and `src/texture/` holds the scope and reduction both use. See the
[texture resolver](src/material/xray_material_resolver.rs), the [surface resolver](src/surface/xray_surface_resolver.rs),
the [class rules](src/surface/rule/xray_surface_rule.rs), the [texture scope](src/texture/xray_texture_scope.rs), and
the [draw modes](src/surface/xray_surface_draw.rs).
