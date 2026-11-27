# xrf-game-movement

The player's movement, a system of the game loop (`xrf-game-loop`'s `LoopSystem`): the camera, what the player holds
and how far they looked, and the `MovementMode` that turns that into motion each step: flying, or walking on the
physics world (`xrf-physics`), which its context (`MovementContext`) hands it each step.

- `MovementCamera` stands in the engine's space, looking along `+z` unturned as X-Ray's camera does; `looking_at` and
  `facing` turn and pitch it without roll (`MovementHeading`: yaw from `+z` towards `+x`, pitch down, held short of
  straight up or down), and it blends between ticks for presentation.
- `MovementCommand` is what the system takes between steps: the player's input, already read through their bindings
  (`MovementInput`: an action down or up, the mouse's raw counts, every action let go, stamped with the platform's
  time), the camera's pose as the control endpoint sets it, and the noclip toggle between flying and walking.
- `MovementInputState` holds the actions held, whether sprinting is toggled on (`Sprint` presses toggle it, as Call of
  Pripyat's `g_sprint_toggle on` plays it), and the mouse's counts since a movement mode last took them.
- `FlyMode` is a spectator's free camera through everything: the mouse's counts turn it (right turns right, down looks
  down) at `sensitivity` radians a count; forward and back fly where it looks, sideways along its right, up and down
  straight up and down; `speed` metres a second, times `boost` while accel is held, diagonals no faster.
- `WalkMode` moves the actor as X-Ray's does, from Anomaly's `[actor]`: the control acceleration is `walk_accel` times
  the pace's factors (running unless accel is held, the back factors going backwards, a forward sprint), and the speed
  is held to a tenth of it (`PHMovementControl.cpp`), reached at that acceleration on the ground and barely steered in
  the air (`air_control_param`). Up jumps at `jump_speed`; the eye stands `camera_height_factor` up the box. The actor
  is put in the physics world the first time walking steps (`WalkState`) and taken out when flight steps.

It depends on the loop and the physics world, never on another system.

## Checks

Run `cargo test --locked -p xrf-game-movement` from the repository root.
