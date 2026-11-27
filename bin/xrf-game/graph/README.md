# xrf-game-graph

The game graph (`CGameGraph`): a few thousand vertices over every level, joined by edges, the map offline objects walk
while only the actor's level has its level graph loaded. `GameGraph::of_spawn` reads it once from the spawn's graph
chunk (`xrf-spawn`'s `SpawnGraphsChunk`) into the game's own types; it never changes during a game, so the systems that
read it share one copy.

- `GameVertexId` is X-Ray's sixteen-bit `_GRAPH_ID`, a vertex's place in the graph.
- `GameVertex` holds where a vertex stands on its level (`level_point`, where an object placed at it stands) and in the
  space of every level laid out together (`game_point`), the level node under it, its terrain kind (the four bytes a
  monster's terrain masks are matched against) and its edges.
- `GameEdge` is an edge's far vertex and its length, as the level graph measured it.
- `get_distance` is the straight line between two vertices' game points, which the simulation weighs targets by
  (`utils.graph_distance`).

The cross tables, which map each level's nodes to vertices, are not kept: nothing offline needs them.

`GamePatrols::of_spawn` reads the spawn's patrol paths by name (`CPatrolPathStorage`), each a list of
`GamePatrolPoint`s (position, level node, game vertex): the points scripts send objects to, such as where a story squad
spawns. Their links are not kept: nothing offline walks a patrol yet.

## Checks

Run `cargo test --locked -p xrf-game-graph` from the repository root.
