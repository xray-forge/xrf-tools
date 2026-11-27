# xrf-game-movement

The player's movement, a system of the game loop (`xrf-game-loop`'s `LoopSystem`): the camera, what the player holds
and how far they looked, and the `MovementMode` that turns that into motion each step: fly now, walking on a physics
engine later.

- `MovementCamera` stands in the engine's space, looking along `+z` unturned as X-Ray's camera does; `looking_at` and
  `facing` turn and pitch it without roll (`MovementHeading`: yaw from `+z` towards `+x`, pitch down, held short of
  straight up or down), and it blends between ticks for presentation.
- `MovementCommand` is what the system takes between steps: the player's input, already read through their bindings
  (`MovementInput`: an action down or up, the mouse's raw counts, every action let go, stamped with the platform's
  time), and the camera's pose as the control endpoint sets it.
- `MovementInputState` holds the actions held and the mouse's counts since a movement mode last took them.
- `FlyMode` is a spectator's free camera through everything: the mouse's counts turn it (right turns right, down looks
  down) at `sensitivity` radians a count; forward and back fly where it looks, sideways along its right, up and down
  straight up and down; `speed` metres a second, times `boost` while the fast action is held, diagonals no faster.

It depends on the loop alone, never on another system.

## Checks

Run `cargo test --locked -p xrf-game-movement` from the repository root.
