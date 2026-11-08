# xrf-visual

Turns parsed X-Ray geometry into what a renderer draws: a serializable description plus one binary buffer of attribute
arrays.

- Packs OGF models, a level's sectors, its grass and its lights.
- Bakes skeletal motions.
- Resolves a model's texture and motion dependencies.

It reads through `xrf-ogf`, `xrf-level`, `xrf-skeleton` and `xrf-spawn`. The native renderer and the desktop app's model
and level viewers use what it produces. It does not render; surface alpha and THM material behaviour belong to
[xrf-material](../xrf-material/README.md).

## Prepare a model for a viewer

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_ogf::OgfFile;
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

- A `VisualPackage` pairs one `VisualDescription` with one byte buffer. Keep them together: the description's
  `VisualSection`s are little-endian byte ranges into that buffer, not separate arrays or file offsets.
- `VisualPacker::pack_meshes` packs meshes built in memory the same way, such as the boxes and spheres `VisualShape`
  makes.

## Geometry and draw ranges

**Coordinates.** Packing converts to renderer space by negating `z`, and reverses triangle winding to match.
`convert_vector` and `convert_declared_bounds` apply the same conversion to anything placed beside the model.

**Skipped parts.** Packing never fails. A submesh without drawable geometry becomes `VisualSubmeshContent::Skipped`,
with a `VisualSkipCause` (`Unsupported` storage or `Malformed` geometry) and a reason. Check each submesh; a returned
package does not prove every part is drawable.

**Geometry.** `VisualGeometry` describes positions, normals, tangents, binormals, UVs, indices and optional skinning.

- `detail_levels` lists every range a consumer may draw, finest first and never empty; `get_default_level` picks the
  finest.
- Do not draw the whole index buffer just because it is there: a progressive mesh stores several detail levels in it.
- Geometry drawn as stored is also cut into `clusters`; skinned geometry is not.

**Bounds.** The description keeps both the declared bounds (converted) and the computed bounds of the packed geometry.
Computed bounds are absent when no submesh produced geometry.

**Skinning.** Skinning indices address the model's bone list, with `VisualSkin::LINKS` links per vertex.
`VisualPoser::pose` freezes a skinned package in a rest pose as static geometry.

## Resolve external assets

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayMountMode, XraySearchPlan};
use xrf_visual::{VisualDependencies, VisualPacker};

fn main() -> xrf_error::XrfResult {
  let file = OgfFile::read_from_path::<XRayByteOrder, _>(&"gamedata/meshes/example.ogf")?;
  let package = VisualPacker::pack(&file);
  let search = XraySearchPlan::new()
    .with_root_mode("gamedata", "gamedata", XrayMountMode::Directory)?
    .open()?;
  let dependencies = VisualDependencies::resolve(&package.description, &search);

  println!("First submesh texture: {:?}", dependencies.find_texture(0));

  Ok(())
}
```

- Texture dependencies keep their submesh index and resolve as the renderer does, substituting
  `ed\ed_not_existing_texture` for a missing one.
- Motion references (`.omf` libraries) have no substitute. One containing a mask can resolve to several assets.
- Embedded motions are already in the model and are not external dependencies.
- Resolution never fails as a whole; a reference that cannot be looked up reports a rejected resolution.
- The caller decides which roots the search covers, and in what order.

## Animation

`VisualMotionBaker::bake` takes the skeleton's bones, their bind data, its partitions, and a paired motion definition
and payload. It returns every frame as model-space bone transforms in renderer space, `VisualTransform::FLOATS` floats
each. `VisualMotionBaker::describe` reports the frame count, duration and animated bones without posing a frame.

- Pair motions through the OGF and OMF `get_motions` and `get_motion_by_name` APIs, not their payload labels.
- Each key run is matched to a bone by name, as the engine does. Bones no motion drives keep their bind transforms.
- The frame count is what the decoded keys hold, not the motion's declared count.
- Baking fails when the bind records do not match the bones, or the key payload does not match the partitions.

`VisualMotions::of` keeps what posing needs from an OGF (nothing for one without a bind pose), so a motion costs no
re-read.

- `bake_named` and `describe_named` look a motion up by name in the file's own motions first, then in the OMF files the
  visual's dependencies located.
- `list_names` names every playable motion, in that order.

## Levels

- `SectorPacker` packs everything one level sector reaches into one buffer of attribute arrays and one index array:
  baked geometry once, placed visuals as instance groups, impostors and progressive windows. A visual that reads badly
  is recorded as a `SectorSkip` rather than failing the sector.
- `DetailsPacker` packs a level's grass: its detail models and each planted slot with the collision triangles its
  planting falls on.
- `LightsPacker` collects the lights the engine would light a level with: hanging lamps placed on their bones, zone idle
  lights, and the level file's own lights other than the sun, with colour animations resolved against `lanims.xr`.
- `HemiEstimator` estimates the hemisphere lighting at a point from the level's collision and lights.

## Checks

Run `cargo test --locked -p xrf-visual` from the repository root.

The optional `typescript-bindings` feature derives frontend type metadata; binary geometry still travels separately from
the serializable description.

Sources: the [model packer](src/pack/visual/visual_packer.rs), [package](src/pack/visual/visual_package.rs),
[motion baking](src/pack/visual/motion/visual_motion_baker.rs), [sector packer](src/pack/sector/sector_packer.rs), and
[dependency resolver](src/resolve/visual_dependencies.rs).
