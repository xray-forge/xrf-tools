# xrf-game-movement

The player's movement, a system of the game loop (`xrf-game-loop`'s `LoopSystem`): the camera, and what the player
holds and how far they looked, which a movement mode turns into motion each step. No movement mode moves the camera yet;
fly mode is the first, walking on a physics engine later.

- `MovementCamera` stands in the engine's space, looking along `+z` unturned as X-Ray's camera does; `looking_at`
  turns and pitches it without roll, and it blends between ticks for presentation.
- `MovementCommand` is what the system takes between steps: the player's input, already read through their bindings
  (`MovementInput`: an action down or up, the mouse's raw counts, every action let go, stamped with the platform's
  time), and the camera's pose as the control endpoint sets it.
- `MovementInputState` holds the actions held and the mouse's counts since a movement mode last took them.

It depends on the loop alone, never on another system.

## Checks

Run `cargo test --locked -p xrf-game-movement` from the repository root.
