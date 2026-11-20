# xrf-object-visual

Turns an SDK `.object` editable object into the model a renderer draws: compiled as the Actor Editor's `Export OGF`
writes it, then packed as any game visual is. The desktop app's model viewer and scene editor draw `.object` files
through it.

It joins [xrf-rawdata](../xrf-rawdata/README.md), which compiles the object, and [xrf-visual](../xrf-visual/README.md),
which packs the model and describes its surfaces through [xrf-material](../xrf-material/README.md). The join is a crate
of its own so that renderer builds taking `xrf-visual` do not pull in the compile's shaders, DDS and tangent work.

## Draw an object

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_material::{XraySurfaceResolver, XrayTextureScope};
use xrf_object_visual::ObjectVisual;
use xrf_rawdata::{ObjectCompileOptions, ObjectCompileSource};
use xrf_vfs::{XrayMountMode, XraySearchPlan};
use xrf_visual::VisualSurfaces;

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

- `ObjectVisual::compile` writes the OGF and reads it back before packing, so the renderer gets exactly what the game
  would load. It fails where `ObjectCompiler` refuses the object or the compiled file does not read back.
- `package` is in renderer space. A rigid object stays in its own space, and whoever places it applies its transform,
  converted as `xrf-visual` converts the model. A skeleton packs in its bind pose with its skin.
- `VisualPacker::pack` packs skinned vertices only, so a rigid object's parts pack as `VisualMesh` values, their
  tangents derived from their uvs, under the description the file declares.
- `motions` holds what posing a skeleton needs, and is `None` for an object with no bind pose.
- `VisualSurfaces::describe` answers every submesh in order. Share one `XraySurfaceResolver` across a tree's models:
  one `shaders.xr` answers every surface in it.

## Checks

Run `cargo test --locked -p xrf-object-visual` from the repository root.

Sources: the [object visual](src/object_visual.rs).
