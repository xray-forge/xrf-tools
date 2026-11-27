# xrf-game-alife

A-Life, a system of the game loop (`xrf-game-loop`'s `LoopSystem`): which objects are online near the actor, and,
later, how offline objects live on. Its context (`AlifeContext`) is the object registry (`xrf-game-objects`) and where
the actor stands; it reaches nothing else.

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

`Alife` runs the sweep at its first step, then once an update interval of simulated time.

## Checks

Run `cargo test --locked -p xrf-game-alife` from the repository root.
