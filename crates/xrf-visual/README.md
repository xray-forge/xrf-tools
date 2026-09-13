# xrf-visual

Converts parsed OGF models into viewer descriptions and binary geometry buffers, and resolves their texture and motion
dependencies.

## Prepare a model for a viewer

```rust,no_run
use xrf_db::{OgfFile, XRayByteOrder};
use xrf_visual::VisualPacker;

fn main() -> xrf_error::XrfResult {
  let file = OgfFile::read_from_path::<XRayByteOrder, _>(&"gamedata/meshes/example.ogf")?;
  let package = VisualPacker::pack(&file);

  assert_eq!(package.description.buffer_length as usize, package.buffer.len());

  println!(
    "{} submeshes, {} bones",
    package.description.submeshes.len(),
    package.description.bones.len()
  );

  Ok(())
}
```

A package pairs one `VisualDescription` with one byte buffer. Keep them together: the description's sections are byte
ranges into that buffer, not independent arrays or file offsets.

## Geometry and draw ranges

Read an `OgfFile` with [xrf-db](../xrf-db/README.md), then call `VisualPacker::pack(&file)` to obtain a `VisualPackage`.
Use `VisualDependencies::resolve` with its description and an `XrayProbe` to locate external textures and motions.
`bake_motion` prepares skeletal animation poses.

Packing converts coordinates and triangle winding for renderer space. A submesh without drawable geometry becomes
`VisualSubmeshContent::Skipped` with a reason, so inspect each submesh rather than treating a returned package as proof
that every part is drawable.

`VisualGeometry` describes positions, normals, tangents, binormals, UVs, indices, and optional skinning data.
Progressive meshes expose valid detail ranges in finest-first order; `get_default_level` selects the finest one.
Do not draw the entire index buffer merely because it is present: it may contain several detail levels.

The description retains both converted declared bounds and computed bounds. Computed bounds are absent when no submesh
produced geometry. Skinning indices address the model's bone list, with four slots per vertex.

## Resolve external assets

```rust,no_run
use xrf_db::{OgfFile, XRayByteOrder};
use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayVfs};
use xrf_visual::{VisualDependencies, VisualPacker};

fn main() -> xrf_error::XrfResult {
  let file = OgfFile::read_from_path::<XRayByteOrder, _>(&"gamedata/meshes/example.ogf")?;
  let package = VisualPacker::pack(&file);
  let vfs = XrayVfs::open(XrayMountMode::Directory, "gamedata")?;
  let probe = vfs.probe().with_step("gamedata", XrayLookupScope::all());
  let dependencies = VisualDependencies::resolve(&package.description, &probe);

  println!("First submesh texture: {:?}", dependencies.find_texture(0));

  Ok(())
}
```

Texture dependency records retain the submesh index; motion references can resolve to several assets when they contain
a mask. Embedded motions are already in the model and are not external dependencies. The caller decides which roots
the probe searches and in what order.

## Animation and limitations

`bake_motion` takes the skeleton, bind data, partitions, and a paired motion definition and payload. Pair motions through
the OGF/OMF APIs rather than their payload labels. Bones not driven by a motion retain their bind transforms.

Skipped submeshes distinguish unsupported geometry from malformed geometry through `VisualSkipCause`. Packing reports
these as data so the rest of the model remains available; parsing and motion baking can still return errors.

This crate produces data; it does not render a scene. Surface alpha and THM material behavior belong to
[xrf-material](../xrf-material/README.md).

## Checks

Run `cargo test --locked -p xrf-visual` from the repository root. The optional `typescript-bindings` feature adds frontend
type metadata; binary geometry still travels separately from the serializable description.

See the [packer](src/pack/visual_packer.rs), [package](src/pack/visual_package.rs), and
[dependency resolver](src/resolve/visual_dependencies.rs).
