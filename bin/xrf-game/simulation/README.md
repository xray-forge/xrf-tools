# xrf-game-simulation

Call of Pripyat's simulation rules, which its scripts hold (`sim_board.script`, `smart_terrain.script`,
`sim_squad_scripted.script`, `simulation_objects.script`): smart terrains spawning squads, squads choosing targets.
It is a library, not a system of the loop: A-Life (`xrf-game-alife`) runs it as the offline behaviour of the objects
its schedule takes. The logic is vanilla's whatever the tree; what a tree's configs set is read from that tree.

## Config

`SimulationConfig::read` reads, once a game start, through `xrf-game-config`'s `ConfigSource` (the tree's `system.ltx`
and any config under `configs`, resolved in the tree's dialect), each section by its `ConfigReader`:

- the squads a new game starts with (`StartSquad`), from `misc\simulation.ltx`: vanilla's `[start_position_<level>]`
  sections, a level at a time in the graph's order, each line a squad and the smarts it is made at; a tree with none of
  them writes Anomaly's form, a section per smart, each line a squad and how many;
- every squad section the start and the respawns name (`SquadDescription`, `squad_descr*.ltx` through `system.ltx`):
  faction, fixed and weighted random members and how many (`SquadSize`), spawn point, script target, behaviour; and
  the class of every squad and member section (`get_class`, the `class` an object of it is created as);
- how each behaviour weighs targets (`SquadBehaviour`, `misc\squad_behaviours.ltx`) and what each simulation object
  offers (`SimulationProps`, `misc\simulation_objects_props.ltx`, with its `sim_avail` condlist);
- each smart terrain's config (`SmartConfig`), from its custom data or the file its `cfg` names: kind, squad number,
  population condlist, respawn sections (`SmartRespawn`), respawn radius and idle time, arrival distance. The input is
  `SmartSpawn::of_spawn`, every `se_smart_terrain` the spawn places.

A section's lines are taken in the engine's order: `CInifile` keeps a section's keys sorted, a key written twice once,
so `r_line` walks them alphabetically (`xrCore/Xr_ini.cpp`, `insert_item`). Vanilla Zaton writes 37 start lines, one
key four times and another twice, so it starts with 33 squads: the base camp's novice squads collapse into the last
line, at `zat_sim_12`.

What does not read (a squad section `system.ltx` lacks, a member it names, a class the game does not know, a condlist,
a smart's missing `cfg`) is a `ConfigIssue` naming its file, section and key (`get_issues`) and is left out; the game
logs them. A behaviour weight or a prop that is no number is left out alone, the rest of its section kept. Fixed from vanilla: the start
fill skips a level with no section rather than stopping at it; a smart's `respawn_radius` is read and kept rather than
replaced by a constant.

## Condlists

`SimulationFacts::pick` judges a condlist as `xr_logic.pick_section_from_condlist` does: the first branch whose
conditions all hold gives its result. An info portion holds by the facts' set (empty in a new game); a probability
(`~50`) holds while one draw, made at the pick's first probability and shared by every later one, is at most it, as
X-Ray draws `math.random(100)` once a pick; a condition function is the hook registry's entry
`xr_conditions.<name>` (`SimulationFunctions`, `xrf-game-hooks`), called with the parameters written after its name,
as X-Ray's scripts look it up in their `xr_conditions` table. The game implements none yet, so one nobody registered
answers false, where X-Ray aborts. A branch's effects are not run.

## The start

`SimulationStart::create` makes a new game's start squads (`sim_board:fill_start_position`, `create_squad`,
`sim_squad_scripted:create_npc`), in the configs' order:

- the squad is created at its smart terrain, assigned to it, as `ON_OFF_S` switching as a group;
- its spawn point is picked (`self`, the smart, or a patrol's first point from `xrf-game-graph`'s `GamePatrols`);
- its fixed members are created once each, then `npc_in_squad` drawn from its random members, a name listed twice
  drawn twice as often; each member is created at the spawn point, alive, under A-Life's control, and joins the squad,
  which then stands where its first member does.

Ids go out as the registry hands them (`ObjectRegistry::create`): the squad, its members, the next squad; each is
placed, grouped and given its rows through the registry's methods. Every draw is `xrf-random`'s at the game's seed
and the squad's id (domains `simulation.squad.spawn_point`, `simulation.squad.members`), so a seed starts the same game
whatever else draws. Fixed from vanilla: a spawn point naming a patrol the spawn lacks puts the squad at its smart,
logged, where vanilla aborts. Not yet: a member's character profile, rank and inventory (`spawn_supplies`), its team
and squad numbers (`setup_squad_and_group`), and the smart's population count (`enter_smart`), which targets and
respawn read.

## Checks

Run `cargo test --locked -p xrf-game-simulation` from the repository root.
