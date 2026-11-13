# xrf-level-compiler

Compiles a level from the files `level build` writes (`xrf-rawdata`'s `LevelBuildExporter`), step by step, into what the
game loads: the geometry with its lightmaps and collision, the details' light and the AI map.

| Step       | Reads                                                              | Runs today                              |
| ---------- | ------------------------------------------------------------------ | --------------------------------------- |
| `geometry` | `build.prj`                                                        | the SDK's xrLC                          |
| `details`  | `build.prj`, `build.details`, `build.cform`, `build.lights`        | this crate's lighter (`DetailLighter`)  |
| `ai`       | `build.aimap`; with full covers also `build.prj` and `build.cform` | this crate's compiler (`AiMapCompiler`) |

`LevelCompiler::compile` takes a `LevelCompileRequest`: the level's directory, the steps (every one when none), the
SDK's `editors` directory for the step the SDK runs (`LevelCompileStep::is_sdk`, the geometry), the game data whose
`gamemtl.xr` and `shaders_xrlc.xr` the compilers read, the textures roots (xrLC takes exactly one), a working directory
for xrLC, whether `level.details` and `level.ai` may be overwritten, the rays' backend (`RayBackend`) and DXC's library
for it, and the AI map's covers. It checks every step's inputs and program first, and a `.thm` under the textures roots
for every texture `build.prj` names where a step reads them, so a missing file stops it before a long run. Then it runs
the steps in order: xrLC in the working directory, under an `fsgame.ltx` it writes there, reporting its last log line
to an `xrf-job` `JobHandle` and killing it when the job is cancelled; the details (`DetailStep`) and the AI map
(`AiStep`) in process, each writing its output staged. Each step's report says what ran it (`LevelCompileStepRun`): an
SDK program and its log, the lighter with the slots it met and the GPU that traced, or the AI map compiler with its
covers, GPU, identity and graph check. `xrf-cli level compile` and the app's level compiler call it.

Each SDK compiler ends a run on a "Congratulation!" message box and waits on it, even once answered; xrLC runs with
`-silent`, which skips it, so it exits on its own.

The spawn (xrAI `-s`) is not a step: it works on every level of a game at once.

## The root spawn builder

The root spawn builder (`spawn/`), xrAI `-s` without xrAI: `SpawnRootBuilder::build` takes each level as a
`SpawnRootLevel` (what its graph merges from, its `level.spawn` and, where it has one, its `level.game`) and an optional
start level, and gives the root spawn (`SpawnRootBuild`: the `SpawnFile` and the connections it left out).

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
  its node toward the other point gets there (`find_straight_walk`), else the length of xrAI's level path search,
  smoothed by straight walks. The search is xrAI's A* kept rule for rule, ties included, since its path decides the
  weight: a step costs a cell, the estimate is twice the cells across and along from the node being expanded, and the
  open list is 8,192 buckets each sorted by cost, a new node ahead of equal ones. Then of every triangle of edges the
  longest pair goes, the longest first.

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

## The AI map compiler

`AiMapCompiler::compile` compiles `build.aimap` into `level.ai` as SDK 0.7's `xrAI -f` does. It takes an `AiMapRequest`
(the map, the covers, the ray model they are traced against, and the `level.ai` it replaces, if any) and an `xrf-job`
`JobHandle`, and gives an `AiMapCompilation`: the `AiMap`, header and nodes, and where its identity came from
(`AiMapGuid`). The `ai` step (`AiStep`) runs it on a level directory: the replaced `level.ai` read for its identity,
the compiled map checked as a graph (`xrf-level`'s `LevelAiGraphVerifier`) and refused with a node linked to itself
before it is written.

- Each node is loaded as xrAI loads it: its cell through the packed grid over the map's box and back (`AiGrid`), its
  plane normal decompressed and normalised.
- `AiMapCovers::None` stores every cover as 0, as xrAI's `-draft` does. `Full` needs an `AiRayModel`, a tracer over
  `build.cform` and its shadow materials, and computes the covers as xrAI's `xrCover` does (`ai/cover/`), in single
  precision and the engine's order:
  - the corner nodes, where a link is missing beside a missing link or a neighbour reaching round (a straight edge's
    nodes are not ones), each trace a ray toward every node linked to them within 30 m, from 1.5 and 0.6 m up; each
    of eight sectors keeps the share its rays let through, and each of four covers averages four sectors;
  - every other node mixes the corner nodes not open both ways that the engine's quadtree lists within 30 m and its
    straight walk reaches, weighted by one over distance; with none, it is open.

  Every ray is a full query, never the SDK's remembered blocker, so the covers are the same on any thread count. Corner
  nodes and other nodes run over the installed rayon pool, a progress unit each; a cancelled job stops with
  `XrfError::Cancelled`.

- The corner rays are traced by the request's `backend`, the detail lighter's `RayBackend` with its `dxc` and
  `gpu`. On the GPU (the `gpu` feature, `ai/cover/gpu/`) nodes are flooded and their rays made on the pool a batch at a
  time while the GPU traces the batch before, through the detail lighter's ray queries one ray at a time; the sums stay
  on the CPU. `AiMapCompilation::gpu` names the GPU that traced.
- The save: the box around every node, its height grown by `EPS_L`; each node's cell, height, plane and covers
  compressed over it; a stable sort by packed cell with the links renumbered; the light 15, which no loader reads.
- The identity is the replaced map's while its graph stands (as many nodes, the same cell size, each at the same cell
  with the same links), so the existing spawn still loads the level; otherwise the first 16 bytes of the SHA-256 of the
  map with a nil identity, so the same input gives the same file. xrAI draws a random one every run.

On the kept SDK runs of `labx8` and `jupiter` a draft compile is the SDK's `-draft` file byte for byte but for the
identity and each node's light. A full compile is the full run's byte for byte on `labx8`; on `jupiter` every cover
value is within one step, 99.97% exact, the rest single rays that differ and the mix carrying them, on either backend;
the GPU compiles `jupiter` in about 10 s, the CPU in 35.

## Shadow materials

`ShadowMaterials::load` reads how each face of xrLC's ray model (`build.cform`) meets a light ray, as the SDK's light
compilers read it. It takes a `ShadowMaterialsRequest`: `build.prj`, the ray model's faces, `shaders_xrlc.xr` and the
texture roots, searched in order.

| Class    | When                                                                                             |
| -------- | ------------------------------------------------------------------------------------------------ |
| `Passes` | the material's compiler shader casts no shadow                                                   |
| `Alpha`  | the texture's `.thm` gives an alpha channel, and it is neither a `lod\` texture nor `level_lods` |
| `Opaque` | anything else                                                                                    |

An alpha face's image is decoded with `xrf-dds`' `decode_sdk_alpha`, and only for alpha textures a face names.
`get_transmittance` answers a tracer hit (`xrf-geometry`'s `TraceHit`, faces by index):

- 1 for a face that passes the ray;
- 0 for an opaque face;
- `1 - alpha` at the nearest texel of the hit's texture coordinates, wrapped, for an alpha face.

Loading fails where the SDK's compilers stop: a texture with no `.thm`, a compiler shader the library lacks, a face of a
material with no compiler shader, or an alpha image that is missing or not the size its `.thm` gives.

## The detail lighter

`DetailLighter::light` lights `build.details` as xrDO_Light does. It takes a `DetailLightRequest` (the details, a tracer
over `build.cform`, the shadow materials, `build.lights`, the backend and DXC's library) and an `xrf-job` `JobHandle`,
and gives a `DetailLighting`: the `level.details` to write, how many slots it met each way, and the GPU that traced.

- Every slot's corners whose densities are all zero are cleared; a slot planting nothing, or touching no face of the
  ray model in its box, is left unlit.
- A planted slot is lit from 15 by 15 points, each dropped onto the highest face under it, by every light of
  `build.lights` reaching the slot: the static lights' colour by the cosine and attenuation, the sun and the sky as
  plain amounts. Each light is scaled by the product of the transmittances its shadow ray crosses.
- A slot is planned (`DetailSlotPlan`): its points' ray origins, each directional light's rays as a packet (its
  direction and the points it faces, by bit), each positioned light's rays one by one, and what each ray's light is
  worth. Its rays are made and traced by the backend, then the amounts summed in the SDK's order, point by point.
- The sums are averaged, halved and stored as four bits each (`LevelDetailsLight::quantize`).

Every shadow ray is a full query: the SDK's lighter first tests the face that last blocked each light, from either
side, so its result depends on the order slots run in and can come out darker; this one is the same on any thread
count. On the kept SDK runs of `mp_darkvalley` and `jupiter` the CPU matches the SDK's slot for slot but for one in
about 20,000 to 90,000, each one step off: the SDK's cache, or a sum under the x87's precision.

| Backend (`RayBackend`) | Traces                                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------------ |
| `Auto`, the default    | the GPU where this build and the machine can, the CPU otherwise                            |
| `Gpu`                  | slots planned on the pool a batch at a time, their rays made and traced with ray queries   |
| `Cpu`                  | each slot whole on a thread of the installed rayon pool, a packet at a time; the reference |

The GPU backend is the `gpu` feature, off by default, so a build without it links no graphics code; the CLI and the
app turn it on. It starts an `xrf-renderer-core` `RayQueryDevice`: D3D12 where DXC's library is given, else Vulkan; on
the GPU the request names (`RayGpu`, the app's renderer choice), else the high-performance one. The ray model goes up once as two
geometries: the opaque faces, which the hardware takes alone and which end a ray at the first; and the alpha faces, which
the shader dims the light through (`detail/gpu/detail_shadow_rays.wgsl`). Faces passing light are left out. A packet's
rays are made on the GPU from its slot's origins, a workgroup a packet, so a slot sends about 20 KB rather than its
rays. On the kept runs it matches the CPU on every `mp_darkvalley` slot and all but 9 of `jupiter`'s 345,801, each one
step off.

The slots run a progress unit each, and a cancelled job stops the lighting with `XrfError::Cancelled`.

The spawn tests take a strip of nodes with a second floor and a cut-off node: the cross table's ties, reach and
unreached nodes, `find_node` between floors, and the graph points xrAI leaves out. The game graph tests take open and
walled grids: bordering regions joined, a triangle's longest side dropped, a wall weighed by the path around it, and
regions apart left unjoined. The merge tests join two levels listed out of id order: the order, offsets, connection
edges and a skipped one, the cross tables' numbering, the level points, the GUID, a missing connection refused, and
the runtime's first draws. The root spawn tests build two such levels with an actor, zones, level changers and
patrols: the objects' order and numbering, the changers' links, the zones' offsets, the patrols' order and a point
over a hole, the actor moved only to another start level, the spawn GUID, and the refusals (no actor or two, a changer
leading nowhere built or to a point its level lacks, two patrols of a name, an unknown start level). They take the
objects' `new_mock` constructors through `xrf-spawn`'s `mocks` feature. The AI map tests compile a generated
four-node map: the sort and renumbering, the header, three planes whose round
trip xrAI's own run gave, the identity rule, and the refusals. The cover tests take a grid: the sectors, which nodes are
corners, the walk stopped by a hole, the quadtree's reach, and a wall whose front blocks the corners' rays and whose
mix reaches the other nodes.

## Develop

```bash
cargo test -p xrf-level-compiler
cargo test -p xrf-level-compiler --features gpu
```

The tests run no SDK compiler. A run of the SDK's compilers on a level is the CLI's or the app's, on copies of a level
outside any game installation. The GPU tests, the detail lighter's and the AI map's covers against the CPU's, trace on
Vulkan, and on D3D12 with the checkout's `.sdks/dxc`; a backend no GPU of the host traces rays on is skipped.
