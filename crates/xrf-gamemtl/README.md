# xrf-gamemtl

Reads and writes `gamemtl.xr`, the game material library:

- **materials**, the physical surfaces a collision face names;
- **pairs**, which decide what happens when two materials meet: step and impact sounds, particles and marks.

`GameMtlFile` holds the library. It is used by the desktop app's archive describer and grass packing (which asks which
materials are passable), by `xrf-cli gamemtl info`, and by [xrf-gamedata](../xrf-gamedata/README.md), which checks the
library and everything resolved against it.

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

- Read with `read_from_path`, `read_from_file`, `read_from_bytes` (for an archived entry), or `read_from_chunk`.
  `FILE_NAME` is where the engine reads it under `$game_data$`.
- `write` emits the whole library as top-level chunks in the writer's buffer.
- `GameMtlFile::new`, `GameMtlMaterial::new` and `GameMtlPair::new` build what the SDK editor would: numbering continues
  from the highest numbers, a material has `SGameMtl`'s defaults with every optional chunk present, and a pair inherits
  from nothing and lists nothing.

## The library

The library is a chunk tree: a version (`0x1000`), the editor's next material and pair numbers (`0x1001`), then a
materials chunk and a pairs chunk with one child each.

- Only version 1, `GAMEMTL_VERSION_COP`, is read; the engine reads no other.
- `material_index` and `material_pair_index` are one past the highest number ever assigned, not counts.

Every game wrote version 1, so `get_game_version` tells games apart as `CGameMtlLibrary::Load` does, by the newest chunk
any material carries:

| Newest chunk present     | Game                |
| ------------------------ | ------------------- |
| Density                  | Clear Sky           |
| Multiplayer shoot factor | Call of Pripyat     |
| Neither                  | Shadow of Chernobyl |

The game reads ammunition, artefacts, bone protection and stalker fire differently depending on it.

## Materials

`GameMtlMaterial` is the engine's `SGameMtl`.

- Its `id` is the library's number for it, which level collision faces store instead of the name.
- `find_material` looks one up by id. `find_material_by_name` looks one up by name ignoring ASCII case, as the engine
  resolves a bone's or a section's material.

Chunks:

- The main, flags, physics and factors chunks are required.
- Description, flotation, injurious speed, density, multiplayer shoot factor and acoustics are optional and stay `None`
  when absent.
- The `get_*` accessors answer what the engine uses instead: `SGameMtl`'s defaults (flotation 1, injurious speed and
  density 0), the single-player shoot factor for the multiplayer one, and for acoustics the
  `GameMtlAcoustics::PREDEFINED` entry for the exact name, or else `default`'s, absorbing as much as the material
  occludes. No shipped library carries acoustics.

Flags:

- `FLAG_*` are the sixteen bits the engine names, `FLAG_NAMES` their labels; `get_named_flags` and `is_passable` read
  them.
- `get_dynamic_materials_count` counts the materials moving objects and bones are made of.
- The editor recomputes slow down, shootable, transparent and injurious from the factors on every save. This crate keeps
  the stored bits as they are.

## Pairs

`GameMtlPair` is the engine's `SGameMtlPair`: two material numbers, the pair's own number, a parent pair
(`NO_PARENT_ID` for none), the `PROPERTY_*` bits it sets itself, and its breaking, step and collision sound, particle
and mark lists, as stored strings.

- The editor writes inherited lists into the child on save, so the engine plays the stored strings; only the editor
  reads the parent.
- `get_own_properties` reads the `PROPERTY_*` bits, named in `PROPERTY_NAMES`.
- `split_list` splits a list as the engine counts and trims it, stopping at the first empty entry between two commas.
- `get_list_entries` gathers one list's entries across the library, with how many pairs name each. The engine asserts
  on more than `MAX_SOUNDS` sounds or `MAX_EFFECTS` particles or marks.
- `is_pair` and `find_pair` match two materials in either order.
- `get_pair_label` names a pair as the engine's debug name does.
- `get_inheriting_pairs_count` counts the pairs that name a parent.

## Chunk order

Writes reproduce a shipped library byte for byte:

- A material's chunks are written in `SGameMtl::Save`'s order (not `Load`'s): `0x1000`, description `0x1005`,
  `0x1001`, `0x1002`, `0x1003`, `0x1008`, `0x1004`, `0x1006`, `0x1007`, acoustics `0x1009`. Optional chunks only when
  present.
- A pair is always `0x1000 0x1002 0x1003 0x1005`.
- Materials and pairs are numbered from zero in list order on write.

## Errors

Reads reject:

- a file that cannot be opened;
- a missing version or numbering chunk, or another version;
- a material or pair missing a chunk the engine asserts on;
- a chunk holding more than its fields, which the writer could not give back.

Chunks the engine does not know are skipped, as the engine skips them.

The crate does not resolve sound, particle or mark names to assets, or check the engine's other load-time asserts;
`xrf-gamedata`'s `materials` check does both.

## Checks

Run `cargo test --locked -p xrf-gamemtl` from the repository root.

Sources: the [library](src/gamemtl_file.rs), [material](src/gamemtl_material.rs), [pair](src/gamemtl_pair.rs),
[acoustics](src/gamemtl_acoustics.rs), [game version](src/gamemtl_version.rs), and [format tests](src/tests.rs).
