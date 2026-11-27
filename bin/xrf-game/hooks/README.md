# xrf-game-hooks

The game's hook points: where a system calls its own behaviour through one registry by name, so a mod's script may
replace or wrap it (plan decision 26). Rust builds the core and Lua may override it; no script engine runs yet, and the
registry is the same API a script will use.

- `HookRegistry` holds every point by name. A system declares each point (`HookPoint<H>`: a name, a `HookKind`, the
  function type `H`) with its Rust default at the game's start (`register`), and reads the entry each time it acts
  there (`get`), never calling the default past it. `replace` swaps an entry by name and answers the old one, so the
  new can wrap it; a function of the wrong type is refused. Names made as the game runs (condlist functions) go
  through `register_named` and `find`.
- Kinds: events (told after the fact, in id order), guards (may refuse), modifiers (may change a value), and condlist
  conditions and effects, named after X-Ray's script tables (`xr_conditions.<name>`, `xr_effects.<name>`).
- `EventBuffer` holds what a system tells during one step (`ObjectEvent`: an event about one object). The game's state
  drains it after the step in id order, each object's own events in the order told: to the event hooks, then to the
  loop's queue for presentation.
- Points declared so far: A-Life's switch (`alife.can_switch_online`, `alife.can_switch_offline`, `alife.on_switched`,
  `xrf-game-alife`'s `AlifeHooks`).

## Checks

Run `cargo test --locked -p xrf-game-hooks` from the repository root.
