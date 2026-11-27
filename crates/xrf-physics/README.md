# xrf-physics

The game's physics: [Jolt](https://github.com/jrouwe/JoltPhysics) through `oxijolt`, behind the workspace's own types,
so nothing else names the engine. The choice and its alternatives are in xrf-agents'
`reports/20261127152002-design-options-xrf-game-physics.md`.

- `PhysicsLevel` is a level's collision as the engine collides with it: its `level.cform` as one static mesh, X-Ray's
  clockwise faces turned, the faces of passable game materials left out (bushes; `flPassable` unless also
  `flActorObstacle`, as X-Ray's physics treats them). Building it takes a second or two for a large level, so it is
  built off the game loop.
- `PhysicsWorld` holds the level and the actors walking it, stepped by the game loop's fixed step, under X-Ray's gravity
  of twice Earth's (`default_world_gravity`).
- An actor is Jolt's `CharacterVirtual`, an upright capsule shaped by `PhysicsActorSettings` (X-Ray's standing box,
  mass, 45° slope limit and climb): `move_actor` moves it at a velocity, sliding along walls, walking up steps and
  sticking to the floor going down, and answers where it stands (`PhysicsActorState`, `PhysicsGround`). On a level's
  triangles its round bottom rides about 0.25 m over a ledge beyond its step-up, so a 0.3 m step-up climbs what X-Ray's
  0.5 m climb does.

## The native library

`oxijolt` is built without its `prebuilt` feature, so the build never downloads: `cargo make fetch-sdks`
(`cli/sdks/fetch.mjs`) places oxijolt 1.6.0's own Windows release archive in `.sdks/jolt`, pinned by its hash, and
`JOLTC_LIB_DIR` (`.cargo/config.toml`) points the build there. The archive's manifest must match `oxijolt`'s features
(`debug-renderer`); it is built with AVX2 and FMA. Windows only for now, so the Linux CI job leaves the crate out.

## Checks

Run `cargo test --locked -p xrf-physics` from the repository root.
