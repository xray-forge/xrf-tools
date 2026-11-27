# xrf-game-alife

A-Life, a system of the game loop (`xrf-game-loop`'s `LoopSystem`): which objects are online near the actor, and which
offline objects take their turn to live on. Its context (`AlifeContext`) is the object registry (`xrf-game-objects`) and where
the actor stands; it reaches nothing else. What it reads and never changes (`AlifeData`: the game graph of
`xrf-game-graph`, the simulation's configs of `xrf-game-simulation`) is read once a session and shared.

Parity is with what the player sees of X-Ray's A-Life, not with its code: its rules are kept where they shape play, its
stalls and faults are not. Each part is a module of its own, measured and replaced alone.

## Settings

`AlifeSettings` is `[alife]` (`gamedata/configs/alife.ltx`), read through `xrf-ltx` (`read_ltx`), Call of Pripyat's
values for any key it lacks: `switch_distance` 150 m and `switch_factor` 0.1 give the online distance (135 m) and the
offline distance (165 m); `schedule_min` and `schedule_max` give the update interval as X-Ray's scheduler spaces it
(`CSheduler::ProcessStep`): 265 ms for Call of Pripyat. Its `process_time` is not read: X-Ray compares it, in
microseconds, with seconds, so it never limits anything.

## The switch

`AlifeSwitch::sweep` (`CALifeSwitchManager`, `xrGame/alife_switch_manager.cpp`) weighs every object of the current
level that nothing holds, in id order, once an update:

- an offline object goes online within the online distance of the actor, an online one offline beyond the offline
  distance; between them it stays as it is, so an object at the edge does not flap;
- an object that may not go offline goes online wherever it stands: a space restrictor (zones, smart terrains, level
  changers), a breakable, a climable, a smart cover, a dead creature, an empty ammo box; one that may not go online
  stays offline (`ObjectSwitching`, `ObjectRegistry::may_go_offline`);
- what an object holds goes with it (`ObjectRegistry::bring_online`, `take_offline`).

Not carried over: X-Ray builds the whole client object as it switches one online (`Process_spawn`, on the main thread),
the stall players know from many objects arriving at once. Going online here is a row in the registry's online table,
and what an online object needs is the business of the systems reading it, off this step.

Not yet: squads switch as a group from their members (`CSE_ALifeOnlineOfflineGroup`), and objects on other levels go
offline at a level change; both are left alone for now.

## The schedule

`AlifeSchedule::update` (`CALifeScheduleRegistry`, `xrGame/alife_schedule_registry.h`, walked by
`CSafeMapIterator::update`) updates offline objects of every level, a fixed number an update (`objects_per_update`: 10
for Call of Pripyat, 20 for Anomaly):

- the objects it may update have a row in the registry's schedule table, from their class: stalkers and monsters,
  smart terrains (and Clear Sky's factions), squads (`ObjectScheduleKind`);
- one needs updating while it is offline, and a creature while A-Life steers it and it is alive, a smart terrain while
  A-Life steers it, a squad always (`ObjectRegistry::needs_update`, X-Ray's `need_update`);
- the walk runs in id order from where the last update stopped, wraps round once at most, and never takes an object
  twice in an update; each object taken is stamped with the tick, so what it does offline can read how long it waited.

With many objects each waits its turn, so an offline squad moves at X-Ray's cadence. Not carried over: X-Ray keeps who
needs updating as a list beside the objects, added to and taken from as they switch, die or join a squad, which falls
out of step with them (Anomaly's engine silences the asserts that catch it); here it is read from the tables as the
walk reaches each object. X-Ray's `mtALife` runs the update on another thread beside the scripts; here it is a step of
the loop's thread, the same every run. No wall-time budget either: X-Ray's `process_time` never limits the walk, and
the plan's decision 24 keeps every step on tick time.

Not yet: nothing acts on an update (graph movement, a squad's or a smart terrain's choices come as modules that run
on each object taken), and a squad's members are not left to it, since no squad has members until squads are built.

## The update

`Alife` updates at its first step, then once an update interval of simulated time: the switch sweeps the current level,
then the schedule runs, as `CALifeUpdateManager::update` orders them. With the actor on no level the graph holds, the
switch rests and the schedule runs on.

## Checks

Run `cargo test --locked -p xrf-game-alife` from the repository root.
