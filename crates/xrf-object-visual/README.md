# xrf-object-visual

Turns an SDK `.object` editable object into the model a renderer draws: the object compiled as the Actor Editor's
`Export OGF` writes it, packed as any game visual is, with each submesh's surface described.

It joins three crates that stay apart: [xrf-rawdata](../xrf-rawdata/README.md) compiles the object,
[xrf-visual](../xrf-visual/README.md) packs the model and [xrf-material](../xrf-material/README.md) answers how each
surface draws. `xrf-visual` taking `xrf-rawdata` would pull the compile's shaders, DDS and tangent work into every
renderer build, and `xrf-rawdata` is conversion only, so the join is a crate of its own. The desktop app's model viewer
and its scene editor draw `.object` files through it.

## Draw an object

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_material::{XraySurfaceResolver, XrayTextureScope};
use xrf_object_visual::{ObjectVisual, VisualSurfaces};
use xrf_rawdata::{ObjectCompileOptions, ObjectCompileSource};
use xrf_vfs::{XrayMountMode, XraySearchPlan};

fn main() -> xrf_error::XrfResult {
  let object = EditableObjectFile::read_from_path::<XRayByteOrder, _>(&"rawdata/objects/example.object")?;
  let source = ObjectCompileSource {
    source_file: String::from("example.object"),
    convertor: String::from("example"),
    built_at: 0,
  };
  let visual = ObjectVisual::compile(&object, &source, &ObjectCompileOptions::default())?;
  let search = XraySearchPlan::new()
    .with_root_mode("gamedata", "gamedata", XrayMountMode::Directory)?
    .open()?;
  let resolver = XraySurfaceResolver::open(&search, XrayTextureScope::shared());
  let surfaces = VisualSurfaces::describe(&resolver, &visual.package.description);

  assert_eq!(surfaces.len(), visual.package.description.submeshes.len());

  Ok(())
}
```

- `ObjectVisual::compile` writes the OGF and reads it back before packing, so the renderer is handed exactly what the
  game would load from the compiled file. A rigid object stays in its own space; whoever places it applies its
  transform, converted to renderer space as `xrf-visual` converts the model.
- A rigid object compiles to static vertices, which `VisualPacker::pack` does not pack (it packs skinned ones), so each
  of its parts is packed as a `VisualMesh`, its tangent frame derived from its uvs, and the model described as the file
  declares it. A skeleton packs through `VisualPacker::pack` like any skinned model.
- `ObjectVisual::motions` keeps what posing a skeleton object needs, and is `None` for a static one.
- `VisualSurfaces::describe` answers every submesh in its order, a game visual's as well as a compiled object's; open one
  `XraySurfaceResolver` for many models, since one `shaders.xr` answers every surface in a tree.
- A compile fails where the object does not export, a static object holding no mesh among the cases `ObjectCompiler`
  refuses.

## Checks

Run `cargo test --locked -p xrf-object-visual` from the repository root.

Sources: the [object visual](src/object_visual.rs) and the [surfaces](src/visual_surfaces.rs).
