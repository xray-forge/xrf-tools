# xrf-game-objects

The game's objects: X-Ray's object registry as id-keyed tables, the one store the game's systems share. It is data,
not a system. The game's state owns it and hands each system the tables it reads or writes as that system's context
(`xrf-game-loop`'s `LoopSystem::Context`), never through a lookup, so the compiler proves which system writes what.

- `ObjectId` is X-Ray's sixteen-bit `_OBJECT_ID`, every table's key; `ObjectId::of` reads X-Ray's none (`0xffff`).
- `IdMap` is a table by id: a dense vector over the id space, a lookup an index and iteration in id order, the same
  every run, as a step that must repeat needs (no hash map is iterated in a step).
- `ObjectRecord` is what every object is, whatever system moves it: section, class, position and direction in the
  engine's space, its graph level, game and level vertices, parent, story and spawn ids, and how it switches
  (`ObjectSwitching`, from its class and flags as X-Ray's `can_switch_online` and `can_switch_offline` decide). The
  game's own type, read once from the spawn (`ObjectRecord::of_spawn`); `xrf-spawn`'s packet types stay at that edge.
- What only some objects have is a concern table beside the records, a row for each object it concerns, never a field
  on every record: the creatures (`ObjectCreature`, health), the ammo (`ObjectAmmo`, rounds) and the schedule
  (`ObjectSchedule`, what A-Life may update offline: its kind, whether A-Life steers it, its last update's tick).
  `may_go_offline` reads them (a dead creature or an empty box stays online), and so does `needs_update` (whether A-Life
  updates an object offline now, X-Ray's `need_update`; never a squad's member, which its squad is updated for).
- Squads: the groups table (`ObjectGroup`, a squad's members, the commander first, and its assigned smart terrain) and
  the members table (`ObjectMember`, the squad each member belongs to), joined by `join_group`.
- The online table (`ObjectOnline`, who is online since which tick) and the children index (what each object holds).
  `bring_online` and `take_offline` move an object with everything it holds and a squad with its members, as X-Ray's
  `add_online` moves children and a squad's `switch_online` its members.
- `create` makes an object during a game, offline, under the next id never handed out, as X-Ray's generator hands
  them out before reusing a freed one (`CID_Generator`); `ObjectRecord::name_created` names it as X-Ray does, its
  section and its id padded to four digits. Freed ids are not reused yet: nothing is released yet.
- `ObjectRegistry::of_spawn` builds a new game's registry from `xrf-spawn`'s `SpawnPlacedObjects`: every readable
  object of `all.spawn`; the levels a record's level id names are the game graph's (`xrf-game-graph`), which owns
  them. A spawn packet's own id is not the object's:
  X-Ray hands ids out as a new game creates each object (`xrServer::PerformIDgen`), and nearly every packet of
  `all.spawn` carries the same placeholder (Anomaly's 18,652 collapsed to 2 when keyed by it). So ids go out in the
  spawn's order, each record keeps its `spawn_id`, and a parent is found by spawn id. Whether X-Ray's order is the
  spawn's is not verified; nothing saved depends on it yet. A new game fills the concern tables from the same packets;
  every object starts offline.

## Checks

Run `cargo test --locked -p xrf-game-objects` from the repository root.
