# xrf-thm

Reads, edits and writes `.thm` texture descriptors: the file beside a texture that says how the converter builds it,
what the engine loads it as, and the bump pair beside it; the SDK's `STextureParams` wrapped in an
`ETextureThumbnail`. `ThmFile` holds one. The SDK's other thumbnails share the extension and are read here too.
[xrf-material](../xrf-material/README.md) resolves what a descriptor makes the renderer bind,
[xrf-texture](../xrf-texture/README.md) builds a texture from one, and the CLI's `thm patch-bump`, the texture verifier,
and the desktop app's texture editor and archive describer read and write them.

## Read and write a descriptor

```rust
use xrf_chunk::XRayByteOrder;
use xrf_thm::{ThmBumpChunk, ThmBumpMode, ThmFile, ThmTextureType};

fn main() -> xrf_error::XrfResult {
  let mut descriptor = ThmFile::new_texture();

  descriptor.bump = Some(ThmBumpChunk {
    mode: ThmBumpMode::Use,
    name: "wpn\\wpn_ak74_bump".into(),
    ..ThmBumpChunk::default()
  });

  let bytes = descriptor.write_to_bytes::<XRayByteOrder>()?;
  let restored = ThmFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(restored, descriptor);
  assert_eq!(restored.write_to_bytes::<XRayByteOrder>()?, bytes);
  assert_eq!(restored.used_bump_name(), Some("wpn\\wpn_ak74_bump"));

  // The engine skips a bump map's own descriptor whole, whatever its bump chunk declares.
  descriptor.texture_type = Some(ThmTextureType::BumpMap);
  assert!(!descriptor.is_described_by_engine());

  Ok(())
}
```

Read a descriptor with `read_from_path`, `read_from_file`, `read_from_bytes` for an archived entry, or
`read_from_chunk`. Write one with `write_to_path`, which creates missing parent directories, `write_to_bytes`, or
`write_to` for any writer. `ThmFile::new_texture` is a descriptor with the values the SDK starts a new one at;
`ThmFile::default` has no chunk at all.

`change_texture_type` sets the type the way the SDK's editor does (`STextureParams::OnTypeChange`), not just the
field. A bump map loses its mip maps. A normal map gains mip maps, the Kaiser filter and RGBA, and loses implicit
lighting. A terrain loses its mip maps, is lit implicitly and becomes DXT1. Both of the last two drop the flags only
the converter's colour path reads. `ThmTextureType`'s `get_cleared_flags`, `get_set_flags`, `get_format` and
`get_mip_filter` give each part, so the desktop app's form makes the same change. Assigning `texture_type` directly
changes nothing else.

## Previews

`ThmThumbnailChunk::decode` turns the stored preview into a 128 by 128 RGBA image. It decompresses LZHUF when the
chunk carries the compression mark, and it turns the SDK's layout around: `D3DCOLOR` pixels as B, G, R, A bytes,
bottom row first. `ThmThumbnailChunk::encode` is the reverse and stores the result compressed, as the SDK saves it.
Both refuse a picture or payload of any other size. The 11 previews in Anomaly's descriptors decode to their textures
the right way up.

## Other thumbnails

The SDK writes a `.thm` for objects, object groups and sounds as well as textures. `ThmKind::detect` reads the version
and type chunks and says which reader a file takes. It answers none for a pair no SDK reader accepts. Both chunks
decide it. A group is stamped with the object's type and told apart only by its version. An object's parameter chunk
reuses the texture material chunk's id, so its counts would read as a material if nothing checked the kind first.

| Kind      | Reader          | Version  | Type | Fields                                                                    |
| --------- | --------------- | -------- | ---- | ------------------------------------------------------------------------- |
| `Texture` | `ThmFile`       | `0x0012` | `1`  | As below                                                                  |
| `Object`  | `ThmObjectFile` | `0x0012` | `0`  | Preview, face and vertex counts in `0x0816`                               |
| `Group`   | `ThmGroupFile`  | `0x0001` | `0`  | Preview, the object names in `0x0001`                                     |
| `Sound`   | `ThmSoundFile`  | `0x0014` | `2`  | Quality, distances and game type in `0x1000`; base volume, AI distance    |

Each reader refuses a file of another kind and requires its parameter chunk. `ThmSoundFile` keeps the base volume
(`0x1001`) and AI distance (`0x1002`) chunks optional, because older sounds lack them. `get_base_volume` and
`get_max_ai_distance` fall back as the SDK does, to `1` and to the maximum distance. A file without those chunks is
written back without them. Each reader keeps unknown chunks and trailing bytes as `ThmFile` does. No game ships these
thumbnails; they sit only in the SDK's rawdata, so the readers are checked against the SDK's source and synthetic
files.

## Chunks and preservation

Each known chunk becomes an optional field, `None` when the file lacks it, and an absent chunk is not written back:

| Field                 | Chunk    | Contents                                                                    |
| --------------------- | -------- | --------------------------------------------------------------------------- |
| `version`             | `0x0810` | Descriptor version; the engine accepts only `VERSION` (`0x0012`)            |
| `thumbnail`           | `0x0811` | The preview picture, copied as stored, its compression mark kept            |
| `thumbnail_type`      | `0x0813` | What the thumbnail describes; `1` is a texture                              |
| `texture_param`       | `0x0812` | `ThmTextureParamChunk`: format, flag word, colours, mip filter, size        |
| `texture_type`        | `0x0814` | `ThmTextureType`: image, cube map, bump map, normal map, terrain            |
| `detail`              | `0x0815` | `ThmDetailChunk`: detail texture name and scale                             |
| `material`            | `0x0816` | `ThmMaterialChunk`: lighting model and weight                               |
| `bump`                | `0x0817` | `ThmBumpChunk`: virtual height, `ThmBumpMode`, bump texture name            |
| `ext_normal_map_name` | `0x0818` | A normal map the generator uses instead of deriving one                     |
| `fade_delay`          | `0x0819` | The mip level the converter starts fading from                              |

The writer emits them in that order, which is the SDK's save order rather than numeric order, then every chunk it could
not fold into a field: `extra` keeps unknown IDs and second copies of a known one (the engine reads only the first) as
`ThmExtraChunk`s, and `trailing` keeps bytes a chunk holds past its fields as `ThmTrailingBytes`, written back after
those fields. A descriptor in that chunk order is rewritten byte for byte. Enumerated values the SDK does not name
(`ThmFormat`, `ThmMipFilter`, `ThmTextureType`, `ThmBumpMode`, `ThmMaterial`) are kept as `Unknown(raw)`, and
`ThmTextureFlags::unnamed` reports flag bits no `ThmTextureFlag` names, so they survive too.

## What the engine reads

Only part of a descriptor matters at runtime. `is_described_by_engine` is the engine's type gate: a cube map or bump
map descriptor contributes nothing, however complete its bump chunk, and a file without a type chunk counts as an
image. `used_bump_name` is the bump name only when the mode is `Use` or `UseParallax` and the name is not empty.
`used_detail_usage` is the detail association only when a detail is named and the flag word sets diffuse or bump
detail. `is_supported_version` and `is_texture_thumbnail` answer whether the file is one the engine loads as a texture
descriptor; both accept a file lacking the chunk.

## Patch a bump declaration

`ThmBumpProcessor` rewrites only the bump chunk of a descriptor on disk and copies every other chunk byte for byte:
`patch_bump_name_to_path` repoints the name and keeps the mode, `patch_bump_off_to_path` clears the name and sets the
mode to `None` as the SDK does, and `patch_bump_to_path` sets either. Before writing, it refuses a file that has no
bump chunk, more than one, or that does not reproduce itself when its existing declaration is written back; after
writing, it reads the destination back and, if the declaration differs, restores the source or removes the new file.
A dry run writes nothing. The returned `ThmBumpPatchReport` gives both sizes and the previous name and mode.

## Errors and checks

Reads reject a file that cannot be opened and a known chunk too short for its fields. `ThmFile` does not reject
another version or thumbnail type; ask `is_supported_version` and `is_texture_thumbnail`, or `ThmKind::detect`.

Run `cargo test --locked -p xrf-thm` from the repository root. See the [descriptor](src/thm_file.rs), its
[enumerations](src/texture), the [bump patcher](src/bump/thm_bump_processor.rs), the [object and group](src/object)
and [sound](src/sound) thumbnails, the [chunks](src/chunks), and the [round-trip](src/tests/round_trip.rs) and
[thumbnail](src/tests/thumbnails.rs) tests.
