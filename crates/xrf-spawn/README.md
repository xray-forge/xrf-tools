# xrf-spawn

Reads and writes the game's `.spawn` files:

- **Spawn sets** such as `all.spawn`, which the alife simulation starts a new game from: alife objects, plus the game
  graph, patrols and artefact points they reference.
- **Level spawn lists**, which a level's server spawns on load.

It depends on `xrf-chunk`, `xrf-math` and `xrf-ltx` (for the LTX form a set is unpacked into). Users: the CLI's `spawn`
commands, the desktop app's spawn descriptions and level view, and `xrf-visual`, which places spawned lamps. It
re-exports `XRayByteOrder`, the byte order every reader here takes.

## Two formats share `.spawn`

| Format           | Type             | Shape                                                                     |
| ---------------- | ---------------- | ------------------------------------------------------------------------- |
| Spawn set        | `SpawnFile`      | Header (0), alife objects (1), artefacts (2), patrols (3), game graph (4) |
| Level spawn list | `LevelSpawnFile` | A flat run of chunks from zero, each one `M_SPAWN` packet; no header      |

Most `.spawn` files in a game tree are level lists; only a few have a header. Each reader refuses the other format:
`SpawnFile` requires exactly its five root chunks, and `LevelSpawnFile` requires every chunk to start with an `M_SPAWN`
packet, which a set's header never does.

## Read a spawn set

`SpawnFile::read_from_path` (and `read_from_file`, `read_from_chunk`) reads the whole set and checks the header's object
and level counts against the chunks. The game graph is most of a set's bytes, so read less when you can:

| Method                           | Reads                                                                                                                                                                          |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `read_graphs_from_path`          | Only the game graph (`SpawnGraphsChunk`): levels, vertices, edges, level points, cross tables. Parses no alife object, so it works on sets with classes this crate cannot read |
| `read_level_objects_from_chunks` | The objects one level places, continuing past unreadable ones (each listed with its index and reason), plus the level changers arriving there                                  |
| `read_level_graph_from_chunks`   | One level's game vertices, and the patrols whose first point is on it                                                                                                          |

### Alife objects

- An alife object is chunk-framed and its reader checks it consumed everything, so a misread class fails that object
  instead of corrupting the next.
- `ClsId::from_section` maps a section to its class, which selects the server-class reader (`AlifeObjectInherited`) for
  its payload. Script fields at the end are read from the object's remaining bytes.
- Stored values are kept as written, not normalized the way the engine's `STATE_Read` would. `LastSpawnTime` keeps its
  three states (`Absent`, `Unset`, `Set`) apart.
- An object's client data and an item's upgrade sections are read as the engine reads them.

Coverage:

- Every class Call of Pripyat creates from a spawn has a reader, taken from the engine registry
  (`object_factory_register.cpp`) and the scripts' (`class_registrator.script`). Classes no shipped spawn contains were
  checked against packets a running game wrote for a new object of each.
- A rocket's section maps to `AlifeClass::Unknown`, since `CSE_Temporary` is not an alife object.
- Multiplayer and Shadow of Chernobyl sections (spectator, team base, rat, flesh and spawn groups, the mounted weapon)
  map to no class.

Refused: an object at spawn version 120 or below (Call of Pripyat and Anomaly write 128), and a section with no known
class.

### Save markers

A script class ends its fields with a save marker counting their bytes, which both vanilla and xrf-engine scripts check
on load. Here the marker is not a field: reading checks it, and writing derives it, so an edited level changer hint
still loads.

## Stalkers and monsters

Call of Pripyat spawns its NPCs at runtime from squads, so no shipped spawn set holds one, but an editor can place them.

- A `stalker` section reads as `AlifeStalker`; a monster section (`dog_weak`, `bloodsucker_normal`) as `AlifeMonster`.
- A stalker's spawn packet starts with its trader fields (`AlifeObjectHumanAbstract`), ahead of the alife object and
  creature fields, unlike the actor's.
- Both script classes end with an `OfflineObjectState`: the level vertex and logic section as the strings `se_stalker`
  and `se_monster` write (`None` for their `nil`). The stalker adds its loot byte. Neither closes these with a marker.
- A new object holds `None`, `None` and `0`, as a compiled spawn does.

These tails mirror `STATE_Write` in xrf-engine's `Stalker.ts` and `Monster.ts`, whose own tests pin their write order.
A change on either side must change the other in the same work. The layout was checked against packets the game wrote
for a new stalker and monster.

## Unpack, edit and write

- `export_to_path` unpacks a set into a directory of editable LTX files; `import_from_path` reads it back. Unpacking and
  packing again should reproduce the original bytes.
- `write_to_path` and `write_to` write the binary set.

Writing refuses what the engine would misread:

- header counts that differ from the objects and levels the set holds;
- graph counts that differ from its tables;
- an object whose payload class is not its section's;
- a patrol link to a point the patrol lacks;
- a smart terrain or skeleton declaring state this crate cannot carry;
- a physics state whose opening byte cannot hold it;
- a shotgun loading more shells than a byte counts;
- an actor's dead body whose bones disagree with its counts, or overrun the engine's buffer.

Reading refuses update bytes beyond what the object's class reads, since a rewrite would drop them.

## The update packet

Each object's chunk carries a second packet, which the engine applies over the spawn packet when it creates the object
(`CALifeSimulatorBase::create`). Where both store a field, the update packet's value is what the game uses.

- A payload owns its update fields beside its spawn fields, named `update_*`, following the same inheritance chain as
  its `UPDATE_Read`.
- A class with an empty update packet owns none, and its reader refuses bytes there.
- A payload with update fields reads and writes both packets together (`AlifePacketsReadWrite`).

| Class                   | Update fields                                                                       |
| ----------------------- | ----------------------------------------------------------------------------------- |
| Every item              | `update_state`: the physics state, `None` for the zero byte standing in for none    |
| Physic object           | `update_state`, as an item's, which a single-player game never applies              |
| Outfit, helmet          | `update_condition` after the item's                                                 |
| Torch                   | `update_flags` after the item's: light, night vision and attached                   |
| Stationary machine gun  | `update_working` and `update_enemy_direction`, which a new gun leaves uninitialised |
| Ammo                    | `update_ammo_left` after the item's                                                 |
| Weapon                  | Condition, weapon flags, rounds loaded, addons, ammo type, state and zoom           |
| Magazined weapon        | `update_fire_mode` after the weapon's                                               |
| With a grenade launcher | `update_grenade_mode`, which opens the packet ahead of every inherited field        |
| Shotgun (magazined)     | `update_ammo_ids`, a byte counting the shells loaded, after the magazined weapon's  |
| Creature, crow, phantom | Health, timestamp, flags, position, model and torso angles, team, squad and group   |
| Monster, human          | After those: the game vertices it walks between, and its distances along that edge  |
| Stalker                 | `update_start_dialog` after the monster's                                           |
| Actor                   | Movement state, two `ScaledDirection`s, radiation, weapon, then `update_body`       |

### Physics state

`PhysicsNetState` models the state's mask. Its opening byte packs the item count with whether the body is enabled and
whether each velocity is omitted; an omitted velocity is `None`.

- The engine reads a physic object's empty packet as no state, and a state ending before its closing byte as awake. It
  writes the zero byte and the closing byte. These two shapes, which no shipped spawn contains, are read and written as
  the engine does.
- A quantised field keeps its stored byte; the float the engine reads is a derived accessor (`get_update_condition`).
- The SDK writes some states uninitialised, so their floats are `RawFloat`s that keep their bits. Vanilla's spawn holds
  5,286 floats of `0xffffffff` and NaNs with payloads; LTX and JSON carry those as hex (`0xffffffff`) and every finite
  value as a number.
- The actor's `ActorPhysicsState` is the body behind its count: none, the living body's state, or a dead body's two
  corners with bones quantised between them. It is refused past the 1,024 bytes the engine holds one in.

### Moving an object

`AlifeObject::move_to` moves an object to a position and direction in every packet that stores where it stands, so an
editor need not know which classes repeat it:

- the spawn packet's position;
- a creature's position and torso (pitch from the direction's x, heading from its y, as `STATE_Read` derives them);
- an item's or physic object's physics state;
- the actor's saved body, shifted by as far as the actor moves.

## Read a level spawn list

A list's object carries the version-independent start of `CSE_Abstract::Spawn_Write` (section, name, two script bytes,
position, direction) and keeps the rest of the packet as is in `payload`. A list therefore rewrites byte for byte,
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

## The Level Editor's form

The Level Editor stores each entity of a scene in `spawn.part` as an `[object_N_spawndata]` section: the entity's spawn
packet as numbered fields, one per value (`000001 = 1`, `000002 = "wpn_groza"`, ...). The editor fills them with the
same `Spawn_Write` that writes the binary packet. This crate does the same: each class writes once, through
[xrf-chunk's field traits](../xrf-chunk/README.md#values-as-bytes-and-as-fields), and the writer decides the form. A
`ChunkWriter` gets the packet's bytes; a `ChunkFieldRecorder` gets the fields with their kinds.

- `AlifeObject::write_spawn_packet` and `read_spawn_packet` handle the spawn packet alone, which is all a scene or a
  `level.spawn` keeps, in either form. Reading fills the update fields the way the SDK's spawn builder does when it
  writes `all.spawn`: from the spawn packet and each class's constructor, and zero where the engine leaves a value
  uninitialised.
- `SpawnPacketHeader` writes the state's byte size into bytes only; fields hold `0` there, as the editor writes it.
  Client data has no field form. `AlifeObject` keeps the header's fields at its own top level in JSON and LTX.
- Some classes write fields differently from bytes, as the engine does, and check `is_field_form` for it: a monster's
  `task_reached` is a byte in bytes and a `u16` in fields, a human's equipment and weapon preferences are left out of
  fields, and the level changer's destination position is three floats.
- A scene may lack the fields a script class adds at the end of its packet, because the SDK's scripts skip them when
  loading inside the editor. The actor, level changer and smart terrain then take their script constructor's values
  (`ScriptSaveMarker::read_or_constructed`); a zone's last spawn time reads as `LastSpawnTime::Absent`.
- A new class implements `ChunkFieldReadWrite`, or `AlifePacketsReadWrite` when it also has update fields, and gets
  both forms from it.

Every entity in GSC's own CoP SDK 0.7 scenes reads and writes back field for field, and every Call of Pripyat
`level.spawn` packet of a known section comes back byte for byte through the text, apart from floats the editor's `%f`
rounds. Stalkers, monsters and squads appear in no GSC scene, so their field form follows the engine's source only.

## Checks

Run `cargo test --locked -p xrf-spawn` from the repository root.

Sources: the [spawn set](src/spawn_file.rs), [alife objects](src/chunks/spawn_alife_spawns_chunk.rs),
[level spawn list](src/level_spawn_file.rs), [spawn packet header](src/data/alife/spawn_packet_header.rs), and
[public exports](src/lib.rs).
