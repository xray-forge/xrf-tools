# xrf-gamemtl

Reads and writes `gamemtl.xr`, the game material library: the physical materials a collision face names, and the pairs
that decide what happens when two materials meet (step and impact sounds, particles and marks). `GameMtlFile` holds the
library. The desktop app's archive describer reads it, its level grass packing asks which materials are passable,
`xrf-cli gamemtl info` reports on it, and [xrf-gamedata](../xrf-gamedata/README.md) checks it and everything resolved
against it.

## Read and write a library

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial, GameMtlPair, GameMtlVersion};

fn main() -> xrf_error::XrfResult {
  let mut grass = GameMtlMaterial::new(0, "materials\\grass");
  let mut barrel = GameMtlMaterial::new(1, "objects\\barrel");

  grass.flags = GameMtlMaterial::FLAG_PASSABLE;
  barrel.flags = GameMtlMaterial::FLAG_DYNAMIC;

  let mut pair = GameMtlPair::new(0, 0, 1);

  pair.step_sounds = String::from("material\\grass\\step_1, material\\grass\\step_2");

  let library = GameMtlFile::new(vec![grass, barrel], vec![pair]);
  let mut writer = ChunkWriter::new();

  library.write::<XRayByteOrder>(&mut writer)?;

  let restored = GameMtlFile::read_from_bytes::<XRayByteOrder>(writer.buffer)?;

  assert_eq!(restored, library);
  assert_eq!(restored.material_index, 2);
  assert!(restored.find_material_by_name("Materials\\Grass").is_some_and(GameMtlMaterial::is_passable));
  assert_eq!(
    restored.find_pair(1, 0).map(|pair| restored.get_pair_label(pair)).as_deref(),
    Some("materials\\grass - objects\\barrel")
  );
  assert_eq!(GameMtlPair::split_list(&restored.pairs[0].step_sounds).len(), 2);
  assert_eq!(restored.get_game_version(), GameMtlVersion::CallOfPripyat);

  Ok(())
}
```

Read a library with `read_from_path`, `read_from_file`, `read_from_bytes` for an archived entry, or `read_from_chunk`;
`FILE_NAME` is where the engine reads it under `$game_data$`. `write` emits the whole library as top-level chunks in the
writer's buffer. `GameMtlFile::new`, `GameMtlMaterial::new` and `GameMtlPair::new` build what the SDK editor would: the
library numbers on from its highest numbers, a material holds `SGameMtl`'s defaults with every optional chunk present,
and a pair inherits from nothing and lists nothing.

## Library, materials and pairs

The library is a chunk tree: a version (`0x1000`), the editor's next material and pair numbers (`0x1001`), then a
materials chunk and a pairs chunk holding one child each. Only version 1, `GAMEMTL_VERSION_COP`, is read, as the engine
reads no other. `material_index` and `material_pair_index` are one past the highest number ever handed out, not counts.

Every game wrote version 1, so `get_game_version` tells them apart as `CGameMtlLibrary::Load` does, by the newest chunk
any material carries: density marks Clear Sky, the multiplayer shoot factor Call of Pripyat, and neither Shadow of
Chernobyl. The game reads ammunition, artefacts, bone protection and stalker fire differently by it.

`GameMtlMaterial` is `SGameMtl`. Its `id` is the library's number for it, which a level's collision faces store instead
of the name; `find_material` looks one up by it and `find_material_by_name` by name regardless of ASCII case, as the
engine resolves a bone's or a section's material. The main, flags, physics and factors chunks are required.
Description, flotation, injurious speed, density, the multiplayer shoot factor and acoustics are optional and stay
`None` when absent. The `get_*` accessors answer what the engine plays with instead: `SGameMtl`'s defaults (flotation
1, injurious speed and density 0), the single-player shoot factor for the multiplayer one, and for acoustics the
`GameMtlAcoustics::PREDEFINED` entry for the exact name, or else `default`'s absorbing as much as the material
occludes. No shipped library carries acoustics.

`FLAG_*` are the sixteen flag bits the engine names and `FLAG_NAMES` their labels, read by `get_named_flags` and
`is_passable`; `get_dynamic_materials_count` counts the materials moving objects and bones are made of. The editor
recomputes slow down, shootable, transparent and injurious from the factors on every save; the crate keeps the stored
bits as they are.

`GameMtlPair` is `SGameMtlPair`: two material numbers, the pair's own number, a parent pair (`NO_PARENT_ID` for none),
the `PROPERTY_*` bits it sets itself (`PROPERTY_NAMES`, read by `get_own_properties`), and its breaking, step and
collision sound, particle and mark lists as stored strings. The editor writes inherited lists into the child on save,
so the engine plays the stored strings and only the editor reads the parent. `split_list` gives a list's entries as the
engine counts and trims them, which stops at the first empty entry between two commas, and `get_list_entries` gathers
one list's entries across the library with how many pairs name each; the engine asserts on more than `MAX_SOUNDS`
sounds or `MAX_EFFECTS` particles or marks. `is_pair` and `find_pair` match two materials in either order,
`get_pair_label` names a pair as the engine's debug name does, and `get_inheriting_pairs_count` counts the pairs naming
a parent.

## Chunk order

A material's chunks are written `0x1000`, description `0x1005`, `0x1001`, `0x1002`, `0x1003`, then `0x1008`, `0x1004`,
`0x1006`, `0x1007` and acoustics `0x1009`, each optional one only when present. That is not the order `SGameMtl::Load`
reads them in, but it is the order `SGameMtl::Save` writes them in, so a shipped library read and written back is byte
for byte the file it came from. A pair is always `0x1000 0x1002 0x1003 0x1005`. Materials and pairs are numbered from
zero in list order on write.

## Errors and checks

Reads reject a file that cannot be opened, a missing version or numbering chunk, another version, a material or pair
missing a chunk the engine asserts on, and a chunk holding more than its fields, which the writer could not give back.
Chunks the engine does not know are skipped, as the engine skips them. The crate does not resolve sound, particle or
mark names to assets, nor check a library against the engine's other load-time assertions; `xrf-gamedata`'s
`materials` check does both.

Run `cargo test --locked -p xrf-gamemtl` from the repository root. See the [library](src/gamemtl_file.rs),
[material](src/gamemtl_material.rs), [pair](src/gamemtl_pair.rs), [acoustics](src/gamemtl_acoustics.rs),
[game version](src/gamemtl_version.rs), and [format tests](src/tests.rs).
