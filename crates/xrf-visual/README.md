# xrf-visual

Turns parsed X-Ray geometry into what a renderer draws: a serializable description plus one binary buffer of attribute
arrays. It packs OGF models, a level's sectors, its grass and its lights, bakes skeletal motions, and resolves a model's
texture and motion dependencies. It reads through `xrf-ogf`, `xrf-level`, `xrf-skeleton` and `xrf-spawn`, and the
native renderer and the desktop app's model and level viewers consume what it produces.

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

A `VisualPackage` pairs one `VisualDescription` with one byte buffer. Keep them together: the description's
`VisualSection`s are byte ranges into that buffer, little-endian, not independent arrays or file offsets.
`VisualPacker::pack_meshes` packs meshes built in memory, such as the boxes and spheres `VisualShape` makes, the same
way.

## Geometry and draw ranges

Packing converts coordinates into renderer space, negating `z`, and reverses triangle winding to match. `convert_vector`
and `convert_declared_bounds` apply the same conversion to anything else a caller places beside the model. Packing
itself never fails: a submesh without drawable geometry becomes `VisualSubmeshContent::Skipped` with a
`VisualSkipCause` (`Unsupported` storage or `Malformed` geometry) and a reason, so inspect each submesh rather than
treating a returned package as proof that every part is drawable.

`VisualGeometry` describes positions, normals, tangents, binormals, UVs, indices, and optional skinning data.
`detail_levels` lists every range a consumer may draw, finest first and never empty, and `get_default_level` selects
the finest one. Do not draw the entire index buffer merely because it is present: a progressive mesh stores several
detail levels in it. Geometry drawn as stored is also cut into `clusters`; skinned geometry is not.

The description retains both the declared bounds, converted, and the computed bounds of the packed geometry; computed
bounds are absent when no submesh produced geometry. Skinning indices address the model's bone list, with
`VisualSkin::LINKS` links per vertex. `VisualPoser::pose` stands a skinned package still in a rest pose as static
geometry.

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

Texture dependencies keep the submesh index and resolve as the renderer does, substituting `ed\ed_not_existing_texture`
for a missing one. Motion references (`.omf` libraries) have no substitute, and one can resolve to several assets when
it contains a mask. Embedded motions are already in the model and are not external dependencies. Resolution never fails
as a whole; a reference that cannot be looked up reports a rejected resolution. The caller decides which roots the
search covers and in what order.

## Animation

`VisualMotionBaker::bake` takes the skeleton's bones, their bind data, the skeleton's partitions, and a paired motion
definition and payload, and returns every frame as model-space bone transforms in renderer space,
`VisualTransform::FLOATS` floats each. `VisualMotionBaker::describe` reports the same frame count, duration and animated
bones without posing a frame. Pair motions
through the OGF and OMF `get_motions` and `get_motion_by_name` APIs rather than their payload labels. Each key run is
matched to a bone by name, as the engine resolves it, and bones not driven by a motion keep their bind transforms. The
frame count is what the decoded keys hold, not the motion's declared count. Baking fails when the bind records do not
match the bones or the key payload does not match the partitions.

`VisualMotions::of` keeps what posing a visual needs from its OGF file, or nothing for one without a bind pose, so a
motion costs no re-read. `bake_named` and `describe_named` look a motion up by name in the file's own motions first,
then in the OMF files the visual's dependencies located, and `list_names` names every motion it can play in that order.

## Levels

- `SectorPacker` packs everything one level sector reaches into one buffer of attribute arrays and one index array:
  baked geometry once, placed visuals as instance groups, impostors and progressive windows. A visual that reads badly
  is recorded as a `SectorSkip` rather than failing the sector.
- `DetailsPacker` packs a level's grass: its detail models and each planted slot with the collision triangles its
  planting falls on.
- `LightsPacker` collects the lights the engine would light a level with: its hanging lamps, placed on their bones, zone
  idle lights, and the level file's own lights other than the sun, with colour animations resolved against
  `lanims.xr`.
  `HemiEstimator` estimates the hemisphere lighting at a point from the level's collision and lights.

## Checks

This crate produces data; it does not render a scene. Surface alpha and THM material behavior belong to
[xrf-material](../xrf-material/README.md). The optional `typescript-bindings` feature derives frontend type metadata;
binary geometry still travels separately from the serializable description.

Run `cargo test --locked -p xrf-visual` from the repository root.

See the [model packer](src/pack/visual/visual_packer.rs), [package](src/pack/visual/visual_package.rs),
[motion baking](src/pack/visual/motion/visual_motion_baker.rs), [sector packer](src/pack/sector/sector_packer.rs), and
[dependency resolver](src/resolve/visual_dependencies.rs).
