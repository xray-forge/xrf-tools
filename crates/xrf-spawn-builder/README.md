# xrf-spawn-builder

Builds a game's root spawn as SDK 0.7's `xrAI -s` does, without xrAI: every level's `level.ai`, `level.spawn` and
`level.game` into one spawn with its game graph, cross tables, entities, artefact points and patrols. `xrf-cli spawn
build` and the desktop app's spawn compiler (`spawn|build_spawn`) call it. It is not a step of `xrf-level-compiler`,
since it works on every level of a game at once.

## Build a spawn

```rust,no_run
use std::path::PathBuf;

use xrf_job::JobHandle;
use xrf_ltx::LtxStandardDialect;
use xrf_spawn_builder::{SpawnRootGamedata, SpawnRootOptions, SpawnRootRequest};

fn main() -> xrf_error::XrfResult {
  let request = SpawnRootRequest {
    gamedata: PathBuf::from("gamedata"),
    levels_root: None,
    levels: None,
    destination: None,
    name: String::from("all"),
    options: SpawnRootOptions::default(),
    is_forced: false,
  };
  // A mod built on DLTX patch files is read under `xrf_dltx::DltxDialect`, which the binaries pick.
  let outcome = SpawnRootGamedata::build(&request, &LtxStandardDialect, &JobHandle::inert())?;

  println!("{}: {} objects on {} levels", outcome.path, outcome.objects, outcome.levels.len());

  Ok(())
}
```

`SpawnRootGamedata::build` runs over a gamedata root as xrAI runs over `$game_levels$` and writes `$game_spawn$`. The
`SpawnRootRequest` fields:

- `gamedata`: the root holding `configs/game_levels.ltx`.
- `levels_root`: the levels' directories; `None` for `levels/` under the root.
- `levels`: the levels to build, by directory name; `None` builds every listed level whose directory holds all three
  files.
- `destination`: where `<name>.spawn` is written; `None` for `spawns/` under the root.
- `options`: the level the actor starts on (xrAI's `-start`), and `is_separator_strict`.
- `is_forced`: write over a spawn already there in any case of its name; unforced, the build refuses before reading a
  level.

Each section's class comes from the root's `configs/system.ltx` through `xrf-spawn`'s `SpawnSectionClasses`, read
under the dialect the caller picks (`SpawnRootGamedata::build` takes it; the binaries choose DLTX), or from its
built-in vanilla table where the root has none. Configs that do not read under that dialect, as game data unpacked in
part, leave the table alone, and the report's `spawn.configs` check says why. Levels are read in parallel, with the
same result. The job hears
three stages under `SpawnRootGamedata::PROGRESS_STAGES` (levels read, spawn built, spawn written), and a cancelled build
stops between them. `SpawnRootOutcome` gives the file written, the levels in id order, both GUIDs, the counts and the
report.

`SpawnRootGamedata::list_levels` gives the levels `game_levels.ltx` lists, each `SpawnListedLevel` marked by whether
its directory is complete.

## Refusals and the report

What xrAI asserts is refused, naming the level and the entity:

- a spawn name that is not a plain file name, an unforced build onto an existing spawn;
- no level asked for, one asked for twice or missing from the list, or no listed level complete;
- an object section with no class;
- a level holding no entity, or keeping none of its graph points;
- no actor, or two;
- two entities of one story id, or two patrols of one name;
- a level changer leading to a level not built, or to a graph point its level lacks;
- a start level not built.

What xrAI lets through is reported, one warning per finding:

| Check                | Finding                                                              |
| -------------------- | -------------------------------------------------------------------- |
| `spawn.graph-points` | A graph point a level's graph leaves out, and why                    |
| `spawn.connections`  | A connection to a level outside the build, left out                  |
| `spawn.separators`   | A restrictor xrAI's separator check covers, cutting AI map nodes off |

`is_separator_strict` refuses a separator cut instead of reporting it.

The separator check finds the map's parts once, floods each part a border touches into pieces, and reports the nodes
outside every piece but the largest, with the first one's position. GSC's own check also fires on nodes it counts
twice, and blames whichever side its first node falls on.

## How it builds

| Directory    | What it holds                                                                               |
| ------------ | ------------------------------------------------------------------------------------------- |
| `gamedata/`  | The build over a gamedata root: the level list, each level's files, the request and outcome |
| `map/`       | A level's AI map (`SpawnAiMap`), its path search and its cross table                        |
| `graph/`     | A level's graph points and game graph, its edges weighed along the AI map                   |
| `merge/`     | The levels' graphs merged into one, and GSC's random stream (`SpawnRandom`)                 |
| `entity/`    | A level's entities, artefact points, level changers and patrols placed                      |
| `separator/` | The space restrictors xrAI checks and the nodes their borders cut off                       |
| `root/`      | The build from levels already read, the actor's start and the report                        |

Per level, then across the levels:

1. **AI map.** `SpawnAiMap::find_node` looks a position up as the engine's `CLevelGraph` does: the node of its cell
   nearest at or under it, else nearest over it.
2. **Graph points.** `SpawnGraphPoints::read` takes `level.spawn`'s graph points in order as the level's game vertices,
   each on the node under it. A point within a millimetre of an earlier one, over no node, or on a kept point's node is
   left out with its `SpawnGraphPointRemoval`, as xrAI drops it.
3. **Cross table.** `SpawnCrossTable::build` floods breadth first from each graph point in turn, taking a node only
   where strictly nearer than its owner so far and going on only through nodes it takes. Unreached nodes keep the first
   vertex and `u32::MAX` cells. xrAI keeps a distance row per graph point, gigabytes on a large level; only the owner's
   distance is ever read back, so a few per-node arrays hold the same tables.
4. **Level graph.** `SpawnLevelGameGraph::build` joins each vertex to every vertex whose region borders its own, in the
   order a depth-first walk over its region meets them. An edge weighs the ground distance of a straight walk from its
   node to the other point (`find_straight_walk`, each cell tested by `xrf-geometry`'s `Box2Pick`), else the length of
   xrAI's path search smoothed by straight walks. The search is xrAI's A\* rule for rule, ties included, since its path
   decides the weight: a step costs a cell, the estimate is twice the cells across and along, and the open list is
   8,192 buckets each sorted by cost, a new node ahead of equal ones. Then the longest edge of each triangle is dropped
   both ways, longest first.
5. **Merge.** `SpawnGameGraphMerger::merge` takes the levels in id order, numbers each one's vertices past the earlier
   levels', and places them in the shared world by the level's offset. Each vertex keeps level points: its region's
   nodes shuffled, then the first tenth, at most 255, or all of ten or fewer. The shuffle is GSC's own stream
   (`SpawnRandom`: the MSVC runtime's `rand` from seed 1 and VS2008's `random_shuffle`), which xrAI never seeds, so it
   gives the shipped points. A graph point naming a connection gets an edge to the point it names, weighed by their
   distance in the world; one naming a level outside the build is skipped (`SpawnSkippedConnection`). The graph GUID is
   the first 16 bytes of the SHA-256 of the graph written without one, so the same levels give the same graph.
6. **Entities.** Each level, in id order, has its entities placed as xrAI's `correct_objects` does
   (`SpawnLevelEntities`): an object on the AI map takes the node under it, raised 0.15 m, and moves onto the node where
   it stands outside its cell; any other takes the level's first vertex. Where a cell holds no node, the slow search
   takes the node whose cell edges come nearest; its plane is computed wide, as the 2009 build's x87 code does, or a tie
   between two cells goes the other way. Each anomalous zone floods the nodes within its radius, keeps those inside it,
   shuffles them from the same stream and takes its count; a zone wanting more reads on through what the buffer held, as
   xrAI does. `level.game` ways become patrols, each point on the node under it.
7. **Links.** Every level changer takes the graph point it names on its destination level, and that point's vertex.
   The zones' artefact points go one level after another. Where a start level other than its own is named, the actor
   moves to the first vertex of that level a search of the game graph reaches.
8. **Determinism.** Patrols are sorted by name, a patrol point over no node takes its nearest node's vertex, and the
   spawn's GUID is derived as the graph's. xrAI's own output depends on string addresses, uninitialised memory and a
   random GUID.

## Against the shipped spawn

On all five vanilla Call of Pripyat levels:

- The cross tables equal the shipped ones bit for bit; zaton's 1.85 M nodes take a quarter of a second.
- Each vertex's edges lead where the shipped ones do, in their order. 2,548 of the 2,550 weights match or are a few
  millionths off; two take another path, about half a metre apart.
- The merged graph chunk, 27 MB, is the shipped one but for its GUIDs and those weights' 121 bytes, every level point
  included. The five levels build and merge in a second.
- The whole spawn builds in 8 s: all 6,464 entities in the shipped order with the shipped nodes, vertices, distances,
  positions and links, all 39,584 artefact points, and every patrol point that has a node.

What differs is what xrAI leaves to chance: the GUIDs, the patrols' order, the 282 patrol points with no node, and the
update packets' uninitialised fields.

## Checks

Run `cargo test --locked -p xrf-spawn-builder` from the repository root.

The tests build on open and walled grids, a strip of nodes with a second floor and a cut-off node, and two-level games
listed out of id order with an actor, zones, level changers and patrols. They cover the cross table's ties and reach,
`find_node` between floors, the graph points xrAI leaves out, edge weights around walls, the merge's order, offsets,
connections and first random draws, the placed objects and patrols, the GUIDs, the report, every refusal above, and the
request's defaults. They take the objects' `new_mock` constructors through `xrf-spawn`'s `mocks` feature.
