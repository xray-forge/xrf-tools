# xrf-game-simulation

Call of Pripyat's simulation rules, which its scripts hold (`sim_board.script`, `smart_terrain.script`,
`sim_squad_scripted.script`, `simulation_objects.script`): smart terrains spawning squads, squads choosing targets.
It is a library, not a system of the loop: A-Life (`xrf-game-alife`) runs it as the offline behaviour of the objects
its schedule takes. The logic is vanilla's whatever the tree; what a tree's configs set is read from that tree.

## Config

`SimulationConfig::read` reads, once a session, through a `SimulationConfigSource` (the tree's `system.ltx` and any
config under `configs`, resolved in the tree's dialect):

- the squads a new game starts with (`StartSquad`), from `misc\simulation.ltx`: vanilla's `[start_position_<level>]`
  sections, a level at a time in the graph's order, each line a squad and the smarts it is made at; a tree with none of
  them writes Anomaly's form, a section per smart, each line a squad and how many;
- every squad section the start and the respawns name (`SquadDescription`, `squad_descr*.ltx` through `system.ltx`):
  faction, fixed and weighted random members and how many (`SquadSize`), spawn point, script target, behaviour;
- how each behaviour weighs targets (`SquadBehaviour`, `misc\squad_behaviours.ltx`) and what each simulation object
  offers (`SimulationProps`, `misc\simulation_objects_props.ltx`, with its `sim_avail` condlist);
- each smart terrain's config (`SmartConfig`), from its custom data or the file its `cfg` names: kind, squad number,
  population condlist, respawn sections (`SmartRespawn`), respawn radius and idle time, arrival distance. The input is
  `SmartSpawn::of_spawn`, every `se_smart_terrain` the spawn places.

A section's lines are taken in the engine's order: `CInifile` keeps a section's keys sorted, a key written twice once,
so `r_line` walks them alphabetically (`xrCore/Xr_ini.cpp`, `insert_item`). Vanilla Zaton's start lines repeat four
keys, so it starts with 34 squads, not 38.

What does not read (a squad section `system.ltx` lacks, a member it names, a condlist, a smart's missing `cfg`) is an
issue (`get_issues`) and is left out; the game logs them. Fixed from vanilla: the start fill skips a level with no
section rather than stopping at it; a smart's `respawn_radius` is read and kept rather than replaced by a constant.

## Checks

Run `cargo test --locked -p xrf-game-simulation` from the repository root.
