# xrf-game-objects

The game's objects: X-Ray's object registry as id-keyed tables, the one store the game's systems share. It is data,
not a system. The game's state owns it and hands each system what it reads or writes as that system's context
(`xrf-game-loop`'s `LoopSystem::Context`), never through a lookup.

- `ObjectId` is every table's key, opaque: a 32-bit slot index and a 21-bit generation, 53 bits of a `u64`, so its
  plain number (`get_raw`, read back by `of_raw`) crosses exactly into a Lua number and JSON. Ids order by slot. The
  generation lets a freed slot take another object without an id someone kept naming the newcomer.
- `IdMap` is a table by id: a dense vector over the slots, a lookup an index and iteration in id order, the same every
  run, as a step that must repeat needs (no hash map is iterated in a step). Each row keeps its id, so an id of a
  slot's earlier object finds nothing.
- `ObjectRecord` is what every object is, whatever system moves it: section, class, position and direction in the
  engine's space, its game and level vertices, story and spawn ids, and how it switches (`ObjectSwitching`, from its
  class and flags as X-Ray's `can_switch_online` and `can_switch_offline` decide). The game's own type, read once from
  the spawn (`ObjectRecord::of_spawn`); `xrf-spawn`'s packet types stay at that edge.
- What only some objects have is a concern table beside the records, a row for each object it concerns, never a field
  on every record: the creatures (`ObjectCreature`, health), the ammo (`ObjectAmmo`, rounds) and the schedule
  (`ObjectSchedule`, what A-Life may update offline: its kind, whether A-Life steers it, its last update's tick).
  `may_go_offline` reads them (a dead creature or an empty box stays online), and so does `needs_update` (whether A-Life
  updates an object offline now, X-Ray's `need_update`; never a squad's member, which its squad is updated for).
- Relations are tables with their indexes, never record fields: the level each object stands on, with each level's
  objects in id order (`get_level_objects`, `count_on_level`), and what holds each object, with what each holds
  (`get_parent`, `get_children`). Squads: the groups table (`ObjectGroup`, a squad's members, the commander first, and
  its assigned smart terrain) and the members table (`ObjectMember`, the squad each member belongs to).
- The online table (`ObjectOnline`, who is online since which tick). `bring_online` and `take_offline` move an object
  with everything it holds and a squad with its members, as X-Ray's `add_online` moves children and a squad's
  `switch_online` its members, and answer the objects that switched.

## Writing it

Rows are added and taken, and relations changed, only through the registry's methods, which keep every table to the
objects that exist and every index in step: `create` (an object during a game, offline, held by nothing), `insert` (an
object at the id it holds, as the spawn's and a save's are), `release` (an object with everything it holds: each
leaves its holder and squad, a released squad's members leave it, every row goes), `set_parent` (refusing a loop),
`move_to` (an `ObjectPlace`: position, level, vertices), `form_group`, `join_group` (out of the squad a member was in),
`set_schedule`, `set_creature` and `set_ammo`. A value already in a row of the records or the schedule may be changed
in place (`get_records_mut`, `get_schedules_mut`), as nothing indexes those values.

Ids go out as `ObjectIds` hands them: a freed slot before a new one, the lowest first, a generation on; a slot freed at
its last generation is retired. X-Ray's generator (`CID_Generator`) hands out never-used ids first because its
sixteen-bit ids carry no generation, and still runs out (`Not enough IDs`). `ObjectRecord::name_created` names a
created object as X-Ray does, its section and its slot padded to four digits.

## A new game

`ObjectRegistry::of_spawn` builds a new game's registry from `xrf-spawn`'s `SpawnPlacedObjects`: every readable object
of `all.spawn`; the levels a level id names are the game graph's (`xrf-game-graph`), which owns them. A spawn packet's
own id is not the object's: X-Ray hands ids out as a new game creates each object (`xrServer::PerformIDgen`), and
nearly every packet of `all.spawn` carries the same placeholder (Anomaly's 18,652 collapsed to 2 when keyed by it). So
objects take slots 0 on in the spawn's order, generation 0, each record keeps its `spawn_id`, and a holder is found by
spawn id once all are put. Whether X-Ray's order is the spawn's is not verified; nothing saved depends on it yet. A new
game fills the concern tables from the same packets; every object starts offline.

## Checks

Run `cargo test --locked -p xrf-game-objects` from the repository root.
