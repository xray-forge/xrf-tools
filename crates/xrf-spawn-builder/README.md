# xrf-spawn-builder

Builds a game's root spawn as SDK 0.7's `xrAI -s` does, without xrAI: every level's `level.ai`, `level.spawn` and
`level.game` into one spawn with its game graph, cross tables, entities, artefact points and patrols. It is not a step
of `xrf-level-compiler`, since it works on every level of a game at once.

`SpawnRootGamedata::build` runs it over a gamedata root, as xrAI runs over `$game_levels$` and writes `$game_spawn$`.
It takes a `SpawnRootRequest` (the root holding `configs/game_levels.ltx`, the directory holding the levels, `levels/`
under the root by default, the levels to build, every listed one whose directory holds the three files when none are
named, the directory the spawn is written into, `spawns/` under the root by default, the spawn's name, the
`SpawnRootOptions` and whether it may write over a spawn already there, in any case of its name) and a job it reports its three stages to
and stops between, reads the levels the game lists (`SpawnGameLevels`), the class of each section from the root's
`configs/system.ltx` (`SpawnSectionClasses`, the built-in table of vanilla sections where the root has none, and for a
section the configs do not give a class the tools read), each level's files and graph (`SpawnLevelFiles`, in parallel;
the result is the same), builds the spawn and writes it as `<name>.spawn`, giving a
`SpawnRootOutcome`. `SpawnRootGamedata::list_levels` gives the levels the list names, each marked by whether its
directory is complete (`SpawnListedLevel`). `xrf-cli spawn build` and the app's spawn compiler (`spawn|build_spawn`)
call it.

## The build

`SpawnRootBuilder::build` takes each level as a `SpawnRootLevel` (what its graph merges from, its `level.spawn` with the
classes its sections are read as and, where it has one, its `level.game`) and its `SpawnRootOptions` (the start level, and whether a restrictor cutting the AI
map refuses the build), and gives the root spawn (`SpawnRootBuild`: the `SpawnFile` and a `Report`). What xrAI asserts
is refused, naming the level and the entity: a level of no entity, no actor or two, two entities of a story id, a level
changer leading to no level built or to a graph point its level lacks, two patrols of a name, an unknown start level.
What xrAI lets through is reported: a connection to a level outside the build (`spawn.connections`), and a restrictor
flagged for xrAI's separator check whose border cuts nodes off (`spawn.separators`). The check (`separator/`) finds the
map's parts once, floods each part the border touches into pieces, and reports the nodes outside every piece but the
largest, with the first one's position; GSC's own check also fires on nodes it counts twice and blames the side its
first node happens to fall on.

| Directory    | What it holds                                                                               |
| ------------ | ------------------------------------------------------------------------------------------- |
| `gamedata/`  | The build over a gamedata root: the level list, each level's files, the request and outcome |
| `map/`       | A level's AI map (`SpawnAiMap`), its path search and its cross table                        |
| `graph/`     | A level's graph points and game graph, its edges weighed along the AI map                   |
| `merge/`     | The levels' graphs merged into one, and GSC's random stream (`SpawnRandom`)                 |
| `entity/`    | A level's entities, artefact points, level changers and patrols placed                      |
| `separator/` | The space restrictors xrAI checks and the nodes their borders cut off                       |
| `root/`      | The build from levels already read, the actor's start and the report                        |

- `SpawnAiMap` reads a level's `level.ai` as the engine's `CLevelGraph` looks nodes up: `find_node` gives the node of a
  position's cell nearest at or under it, else nearest over it.
- `SpawnGraphPoints::read` takes `level.spawn`'s graph points in order, the level's game vertices to be, each on the
  node under its position; one within a millimetre of a point before it, over no node, or on a kept point's node is
  left out with its `SpawnGraphPointRemoval`, as xrAI drops it.
- `SpawnCrossTable::build` gives every node its game vertex and distance as xrAI does: each graph point in turn floods
  breadth first from its node, taking a node only where strictly nearer than its owner so far and going on only through
  nodes it takes; nodes no point reaches keep the first vertex and `u32::MAX` cells. xrAI keeps a distance row a graph
  point, gigabytes on a large level; only the owner's distance to each node is ever read back, so a few arrays a node
  hold the same tables.

- `SpawnLevelGameGraph::build` gives each vertex an edge to every vertex whose region its own borders, in the order a
  depth first walk over its region meets them. An edge weighs the distance across the ground where a straight walk from
  its node toward the other point gets there (`find_straight_walk`, each cell tested by `xrf-geometry`'s `Box2Pick`),
  else the length of xrAI's level path search, smoothed by straight walks. The search is xrAI's A* kept rule for rule,
  ties included, since its path decides the weight: a step costs a cell, the estimate is twice the cells across and
  along from the node being expanded, and the open list is 8,192 buckets each sorted by cost, a new node ahead of equal
  ones. Then of every triangle of edges the longest pair goes, the longest first.

- `SpawnGameGraphMerger::merge` takes each level as a `SpawnMergeLevel` (its `game_levels.ltx` entry, AI map, graph
  points, graph and cross table) and gives the root spawn's graph chunk (`SpawnGameGraphMerge`). The levels go in id
  order, each one's vertices numbered past the levels' before it and placed in the shared world by its offset. Each
  vertex keeps level points: its region's nodes shuffled, the first tenth, at most 255, or all of ten or fewer. The
  shuffle is GSC's own stream (`SpawnRandom`: the MSVC runtime's `rand` from seed 1 and VS2008's `random_shuffle`),
  which nothing in xrAI seeds, so it gives the shipped points. A graph point naming a connection gets an edge to the
  point it names, the distance between them across the world; one naming a level outside the build is skipped
  (`SpawnSkippedConnection`). The graph GUID is the first 16 bytes of the SHA-256 of the graph written with none, so
  the same levels give the same graph and any change another.

- Then each level, in id order, has its entities placed (`SpawnLevelEntities`) as xrAI's `correct_objects` does: an
  object standing on the AI map takes the node under it, raised 0.15 m, and moves onto the node where it stands outside
  its cell; any other the level's first vertex. Where a cell holds no node, the slow search takes the node whose cell
  edges come nearest; its plane is worked wide, as the 2009 build's x87 code does, or a tie between two cells goes the
  other way. Each anomalous zone floods the nodes within its radius, keeps those lying within it, shuffles them from the
  same stream, and takes its count of them; a zone wanting more reads on through what the buffer held, as xrAI does.
  The level's `level.game` ways become patrols, each point on the node under it.
- Then every level changer takes the graph point it names on the level it leads to, and that point's vertex; the zones'
  artefact points go one level after another; the one actor moves, where a start level other than its own is named, to
  the first vertex of that level a search of the game graph reaches. Patrols go sorted by name, a point with no node
  under it takes its nearest node's vertex, and the spawn's GUID is derived as the graph's: xrAI's own output depends on
  string addresses, uninitialised memory and a random GUID.

On all five vanilla Call of Pripyat levels the cross tables equal the shipped ones cell for cell, bit for bit, zaton's
1.85 M nodes in a quarter of a second. Each vertex's edges lead where the shipped ones do, in their order; 2,548 of the
2,550 weights are the shipped ones or a few millionths off, and two take another path, about half a metre apart. The
merged graph chunk, 27 MB, is the shipped one but for its GUIDs and those weights' 121 bytes, every level point
included; the five levels build and merge in a second. The whole spawn builds in 8 s: all 6,464 entities in the shipped
order, with the shipped nodes, vertices, distances, positions and links, all 39,584 artefact points, and every patrol
point with a node as shipped. What differs is what xrAI leaves to chance: the GUIDs, the patrols' order, the 282 points
with no node, and the update packets' uninitialised fields.

## Develop

```bash
cargo test -p xrf-spawn-builder
```

The AI map tests take a strip of nodes with a second floor and a cut-off node: the cross table's ties, reach and
unreached nodes, `find_node` between floors, and the graph points xrAI leaves out. The game graph tests take open and
walled grids: bordering regions joined, a triangle's longest side dropped, a wall weighed by the path around it, and
regions apart left unjoined. The merge tests join two levels listed out of id order: the order, offsets, connection
edges and a skipped one, the cross tables' numbering, the level points, the GUID, a missing connection refused, and the
runtime's first draws. The root spawn tests build two such levels with an actor, zones, level changers and patrols: the
objects' order and numbering, the changers' links, the zones' offsets, the patrols' order and a point over a hole, the
actor moved only to another start level, the spawn GUID, the report (a connection left out, a wall across the map and
its strict refusal), and the refusals (no actor or two, a shared story id, a level of no entity, a changer leading
nowhere built or to a point its level lacks, two patrols of a name, an unknown start level). They take the objects'
`new_mock` constructors through `xrf-spawn`'s `mocks` feature. The gamedata tests take a level list and placeholder
level files: the request's default levels and spawn directories, the listing, and the refusals before any level is read
(no level asked for, a name that is no file name, an unlisted level, one asked for twice, none built, the game's own
spawn unforced).
