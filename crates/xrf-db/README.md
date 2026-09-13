# xrf-db

Reads, writes, and inspects X-Ray binary asset formats. Despite the name, `.db` archive volumes belong to
[xrf-archive](../xrf-archive/README.md).

## Choose a format

| Asset                         | Entry point                                  |
| ----------------------------- | -------------------------------------------- |
| OGF models                    | `OgfFile`                                    |
| OMF skeletal motions          | `OmfFile`                                    |
| Spawn files                   | `SpawnFile`                                  |
| Particle libraries            | `ParticlesFile`                              |
| THM texture descriptors       | `ThmFile`                                    |
| `shaders.xr` blender library  | `ShaderLibraryFile`                          |
| Level geometry, AI, collision | `LevelFile`, `LevelAiFile`, `LevelCformFile` |

## Read a model

```rust,no_run
use xrf_db::{OgfFile, XRayByteOrder};

fn main() -> xrf_error::XrfResult {
  let model = OgfFile::read_from_path::<XRayByteOrder, _>(&"gamedata/meshes/example.ogf")?;

  for name in model.get_motion_names() {
    println!("Motion: {name}");
  }

  Ok(())
}
```

For a model read from a VFS or archive, pass the owned bytes to `OgfFile::read_from_bytes::<XRayByteOrder>`.
The format reader receives bytes; mount order and engine reference lookup remain the caller's responsibility.

Start with the format type's read methods. Each format owns its supported versions, chunk checks, and write/export
operations; support for one format does not imply support for every engine fork. The [exports](src/lib.rs) list the
typed chunks and processors for motion, texture-reference, and bump edits.

## Read and write a texture descriptor

```rust
use xrf_db::{ThmFile, XRayByteOrder};

fn main() -> xrf_error::XrfResult {
  let descriptor = ThmFile::new_texture();
  let bytes = descriptor.write_to_bytes::<XRayByteOrder>()?;
  let parsed = ThmFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert!(parsed.is_texture_thumbnail());

  Ok(())
}
```

`ThmFile` describes texture processing and material metadata; DDS pixel data is handled by
[xrf-dds](../xrf-dds/README.md). Reading a THM does not automatically load its texture, bump pair, or detail texture.

## Motions and edits

OGF and OMF expose motion definitions paired with their key payloads through `get_motions` and `get_motion_by_name`.
Use those APIs for identity: the payload's preserved label is not necessarily the motion's lookup name.

Use the dedicated texture-reference, motion-reference, and bump processors for those focused edits. A format's public
read API does not imply that it supports every write or export direction; consult the corresponding file type before
building an editor around it.

Use [xrf-visual](../xrf-visual/README.md) to turn parsed OGF data into viewer buffers, and
[xrf-material](../xrf-material/README.md) to resolve texture and surface behavior.

## Compatibility, features, and checks

Readers validate the chunks and versions they model. A successful read is not a promise of universal fork compatibility
or byte-for-byte reconstruction of all data; preserve the distinction between recognized fields and retained residue.
Test edits on representative assets for the target engine.

The optional `fixtures` feature exposes builders for tests in dependent crates. `typescript-bindings` enables type
metadata for frontend bindings.

Run `cargo test --locked -p xrf-db` from the repository root. See [OGF](src/ogf/ogf_file.rs),
[OMF](src/omf/omf_file.rs), and [THM](src/thm/thm_file.rs) for their exact read and write contracts.
