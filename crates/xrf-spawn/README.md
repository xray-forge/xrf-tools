# xrf-spawn

Reads and writes the `.spawn` formats: spawn sets and the alife records inside them, with the game graph, patrols and
artefact points they reference, and the per-level spawn lists a level ships. A spawn set such as `all.spawn` is what
the engine's alife simulation starts a new game from; a level spawn list is what a level's server spawns on load.

The crate depends on `xrf-chunk`, `xrf-math`, and `xrf-ltx`, which supplies the LTX binding a set is unpacked through.
Consumers are the CLI's `spawn` commands, the desktop app's spawn descriptions and level view, and `xrf-visual`, which
places spawned lamps. It also re-exports `XRayByteOrder`, the byte order every reader here is called with.

## Two formats share `.spawn`

| Format           | Type             | Shape                                                                     |
| ---------------- | ---------------- | ------------------------------------------------------------------------- |
| Spawn set        | `SpawnFile`      | Header (0), alife objects (1), artefacts (2), patrols (3), game graph (4) |
| Level spawn list | `LevelSpawnFile` | A flat run of chunks from zero, each one `M_SPAWN` packet; no header      |

Most `.spawn` files in a game tree are level lists; only a few carry a header. Each reader refuses the other format:
`SpawnFile` requires exactly its five root chunks, and `LevelSpawnFile` requires every chunk to open with an `M_SPAWN`
packet, which a set's header never does.

## Read a spawn set

`SpawnFile::read_from_path` (and `read_from_file`, `read_from_chunk`) reads the whole set and checks that the header's
object and level counts match what the chunks hold. A whole read is large: the game graph is most of a set's bytes. Read
less when less is needed:

- `read_graphs_from_path` reads only the game graph (`SpawnGraphsChunk`): levels, vertices, edges, level points and
  cross tables. It parses no alife object, so it also works on a set holding an object class this crate has no reader
  for.
- `read_level_objects_from_chunks` reads the objects one level places, keeping going past an object it cannot read: the
  result lists each one skipped with its index and reason, and the level changers arriving at that level.
- `read_level_graph_from_chunks` reads one level's game vertices and the patrols whose first point stands on it.

An alife object is chunk-framed and its reader asserts full consumption, so a misread class fails that object rather
than desynchronizing the next one. `ClsId::from_section` maps a section to its class, which gives the object's identity
and so which server-class reader (`AlifeObjectInherited`) parses its payload; scripted fields at the tail are read from
the object's remaining byte budget. An object at spawn version 120 or below is refused (Call of Pripyat and Anomaly
write 128), and so is a section with no known class. Stored values are kept as written rather than normalized the way
the engine's `STATE_Read` would, and `LastSpawnTime` keeps its three states (`Absent`, `Unset`, `Set`) apart. An
object's client data and an item's upgrade sections are read as the engine reads them; its update packet is carried
verbatim, though the engine applies it over the spawn packet when it creates the object.

A script class closes its own fields with a save marker counting their bytes, which both vanilla and xrf-engine
scripts assert on load. The marker is not a field here: reading checks it against what was read, and writing derives
it, so an edited level changer hint still loads.

`export_to_path` unpacks a set into a directory of LTX files a person can edit, and
`import_from_path` reads that directory back; `write_to_path` and `write_to` write the binary set. Unpacking and packing
again is meant to reproduce the original bytes.

Writing refuses what the engine would misread rather than writing it: header counts that differ from the objects and
levels the set holds, graph counts that differ from its tables, an object whose payload class is not its section's, a
patrol link to a point the patrol lacks, and a smart terrain or skeleton declaring state this crate cannot carry.

## Read a level spawn list

A list's object carries the version-independent head of `CSE_Abstract::Spawn_Write` - section, name, two script bytes,
position, and direction - and keeps the rest of the packet verbatim in `payload`, so a list rewrites byte for byte
whatever spawn version or server class wrote it.

```rust
use xrf_chunk::ChunkWriter;
use xrf_math::Vector3d;
use xrf_spawn::{LevelSpawnFile, LevelSpawnObject, XRayByteOrder};

# fn main() -> xrf_error::XrfResult {
let list: LevelSpawnFile = LevelSpawnFile {
  objects: vec![LevelSpawnObject {
    id: 0,
    // `M_SPAWN`, which the reader requires of every packet.
    net_action: 1,
    section: String::from("medkit"),
    name: String::from("medkit_0000"),
    script_game_id: 0,
    script_rp: 0,
    position: Vector3d::new(1.0, 2.0, 3.0),
    direction: Vector3d::new(0.0, 0.0, 0.0),
    // The class-dependent rest of the packet, carried without being read.
    payload: vec![0x7f, 0x00],
  }],
};
let mut writer: ChunkWriter = ChunkWriter::new();

list.write::<XRayByteOrder>(&mut writer)?;

let read: LevelSpawnFile = LevelSpawnFile::read_from_bytes::<XRayByteOrder>(writer.buffer.clone())?;

assert_eq!(read, list);
assert_eq!(read.get_sections().get("medkit"), Some(&1));
# Ok(())
# }
```

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-spawn
```

The README is the crate documentation, so its Rust examples run as doctests. See the [spawn set](src/spawn_file.rs),
[alife objects](src/chunks/spawn_alife_spawns_chunk.rs), [level spawn list](src/level_spawn_file.rs), and
[public exports](src/lib.rs).
