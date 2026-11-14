# xrf-level-compiler

Compiles a level from the files `level build` writes (`xrf-rawdata`'s `LevelBuildExporter`), step by step, into what the
game loads: the geometry with its lightmaps and collision, the details' light and the AI map.

| Step       | Reads                                                              | Runs today                              |
| ---------- | ------------------------------------------------------------------ | --------------------------------------- |
| `geometry` | `build.prj`, `shaders_xrlc.xr`, the textures                       | this crate's (`GeometryStep`), or xrLC  |
| `details`  | `build.prj`, `build.details`, `build.cform`, `build.lights`        | this crate's lighter (`DetailLighter`)  |
| `ai`       | `build.aimap`; with full covers also `build.prj` and `build.cform` | this crate's compiler (`AiMapCompiler`) |

The geometry step is this crate's own (`GeometryLevel::compile`: xrLC's geometry phases, progressive meshes above
draft quality, every vertex and tree lit by `LightHemisphere`, `LightVertices` and `LightModels`, the lightmapped faces
cut into charts, lit and packed onto pages, each implicitly lit texture lit over its own coordinates) unless the
request names the SDK, whose xrLC then compiles and lights it.

## Lightmaps

`chart/` cuts, sizes and lights charts as xrLC does and packs them its own way. `LightChart::cut` is `xrPhase_UVmap`:
per material split, the largest face not yet charted starts a chart, which takes, depth first through the faces sharing
a split vertex, every face within the smoothing angle and a degree of its normal; `LightChart::of` is `OA_Export`, the
projection along the faces' area-weighted normal and the size in texels from the density, between 1 and 510 a side.
`GeometrySplits::cut` detaches each chart into a split, and a split the subdivision cuts is charted again a half at a
time; `GeometrySplits::merge` then joins only splits on one page.

`LightChartLighter` is `CDeflector::Light`: each texel the mean of its jitter samples that fall on a face, lit there
with `LightPoint` facing the corners' normals blended and joined with the face's, the face itself let through; then
the faces' edges walked an eighth of a texel at a time; each light halved and stored as `base_color` does
(`LightmapLayer`). Border rings fill the rest (`ApplyBorders`); a chart within its RMS of one colour becomes it, two
texels a side; else each side tries a half, a quarter or three quarters, kept where `LightmapResampler`'s Lanczos
shrink and cubic growth (`imf_Process`) hold, and the chart is lit again at the smaller size; then it takes its border.
Each light remembers the face that last stopped it on the chart (`LightOccluders`), as `rayTrace` does, which one
triangle test settles before a full query: the light is the same, ten times faster on `labx8`.

`LightChartMask` is what a lit chart takes on a page, its texels lit and two rings, kept as each row's span.
`LightmapAtlas::pack` takes the charts in `xrPhase_MergeLM`'s order and selection and places each on a 1024-texel page
at the first place by row it fits, upright before crosswise, over a bit a texel; a mask of the same spans starts from
where the last one fitted. `LightmapAtlas::compose` lays each chart's texels on its page, fills sixteen border rings and
packs `_1` (static light, sun in alpha) and `_2` (sun, hemisphere in alpha), which the step writes as BC3.

`ImplicitLighter` is `ImplicitLightingExec`, for a texture whose `.thm` says it is lit implicitly, such as terrain's:
a map the texture's size over its own coordinates, each texel the mean of every face under each of its jitter samples,
lit with the chart lighter's `LightPoint`; the rings then fill the map, the texture's alpha takes the hemisphere and
the map the static light and sun (`ImplicitMap`), which the step writes as `<texture>.dds` and `<texture>_lm.dds` in the
level's folder. As xrLC, it lights none at draft quality.

On the SDK's runs the charts hold the same triangles as xrLC's for 99.56 to 100% of them, the pages are as many, the
light, compared in world space, lies within the threshold two SDK bakes of a level differ by, and `jupiter`'s terrain
map within p99 6 steps of xrLC's texel for texel.

`LightmapBackend` lights the charts and the implicit maps by the request's `RayBackend`. On the GPU (`LightGpu`) a
chunk of charts' samples is planned a few hundred points a plan (`LightPlan`) with the lights reaching them, every
light's rays a packet the shader makes from the points (`LightLampRays::Packet`), traced in batches by the shared
`RayGpuTracer` under xrLC's rules (both sides of a face, `1 - alpha²`, the sample's own face passed, and each face the
device meets tested again by the CPU's ray test), finished on the pool; the charts that shrink are lit again together,
then each takes its border. A point's light is the CPU's but where the GPU's arithmetic parts from it in a last bit,
which a chart's compression can carry across it: on the proof levels the pages lie within p90 3, p99 12 steps of the
CPU's, and as close to xrLC's as the CPU's are.

`LevelCompiler::compile` takes a `LevelCompileRequest`: the level's directory, the steps (every one when none), the
SDK's `editors` directory, which runs the geometry by xrLC (`LevelCompileStep::is_sdk`), the game data whose
`gamemtl.xr` and `shaders_xrlc.xr` the compilers read, the textures roots (xrLC takes exactly one), a working directory
for xrLC, whether `level.details` and `level.ai` may be overwritten, the rays' backend (`RayBackend`) and DXC's library
for it, and the AI map's covers. It checks every step's inputs and program first, and a `.thm` under the textures roots
for every texture `build.prj` names where a step reads them, so a missing file stops it before a long run. Then it runs
the steps in order: xrLC in the working directory, under an `fsgame.ltx` it writes there, reporting its last log line
to an `xrf-job` `JobHandle` and killing it when the job is cancelled; the details (`DetailStep`) and the AI map
(`AiStep`) in process, each writing its output staged. Each step's report says what ran it (`LevelCompileStepRun`): an
SDK program and its log, the geometry compiler with its census and the GPU that traced its light, the lighter with the slots it met and the GPU that traced, or the AI map compiler with its
covers, GPU, identity and graph check. `xrf-cli level compile` and the app's level compiler call it.

Each SDK compiler ends a run on a "Congratulation!" message box and waits on it, even once answered; xrLC runs with
`-silent`, which skips it, so it exits on its own.

The spawn (xrAI `-s`) is not a step: it works on every level of a game at once.

## The root spawn builder

The root spawn builder (`spawn/`), xrAI `-s` without xrAI: `SpawnRootBuilder::build` takes each level as a
`SpawnRootLevel` (what its graph merges from, its `level.spawn` and, where it has one, its `level.game`) and its
`SpawnRootOptions` (the start level, and whether a restrictor cutting the AI map refuses the build), and gives the root
spawn (`SpawnRootBuild`: the `SpawnFile` and a `Report`). What xrAI asserts is refused, naming the level and the entity:
a level of no entity, no actor or two, two entities of a story id, a level changer leading to no level built or to a
graph point its level lacks, two patrols of a name, an unknown start level. What xrAI lets through is reported: a
connection to a level outside the build (`spawn.connections`), and a restrictor flagged for xrAI's separator check
whose border cuts nodes off (`spawn.separators`). The check finds the map's parts once, floods each part the border
touches into pieces, and reports the nodes outside every piece but the largest, with the first one's position; GSC's
own check also fires on nodes it counts twice and blames the side its first node happens to fall on.

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

`get_opacity` gives what a hit stops, the alpha alone, for xrLC's `1 - alpha²`.

## `LightPoint`

`LightPoint::evaluate` gathers the light a point with a normal receives from `build.lights` (`LightSource`s by role):
static lights' colour by the cosine, the sun and the sky as amounts, lights at a position fading with distance, each
scaled by what its shadow ray lets through the ray model. Its two variants (`LightPointVariant`) are the two SDK
compilers' copies, which differ in three ways:

| Variant   | Static point light                                  | Faces that shadow | An alpha texel lets through |
| --------- | --------------------------------------------------- | ----------------- | --------------------------- |
| `Details` | over its attenuation (xrDO_Light)                   | front only        | `1 - alpha`                 |
| `Level`   | its attenuation's inverse less its falloff (xrLC)   | either side       | `1 - alpha²`                |

The detail lighter plans its rays through `LightPoint::get_ray`; the light baker's lighters share it. On the SDK's own
vertex light of `l01_test` and `labx8`, `Level` gives xrLC's bytes on 99.99% and 99.93% of vertices.

On it the vertex light is built as xrLC builds it, each term stored as `base_color`'s `fixed16<16>` (`LightColor`):

- `LightHemisphere`: every welded vertex's sky term before the smoothing split, through the ray model of the welded
  mesh (`GeometryRayModel::build_welded`), halved and averaged with its neighbours five times;
- `LightVertices`: each split vertex's sky term, and a vertex-lit one's static light and sun, halved, then moved toward
  the brightest vertex at its place by its translucency;
- `LightModels`: each multi-use model's sky light in its own space, which its trees store, and each placement's light
  in the level, which gives its tree's scale and bias by a least-squares fit; samples are jittered from a seeded source
  where xrLC uses its random one.

`LightQuality` sets what they light by: `Scene`, `build.prj`'s own lights, or `Draft`, the sky and sun the Level Editor
builds at quality 1 (`xrf_geometry::Sky`) in place of the `$hemi` and `$sun` layers' directional lights, the lamps under
them kept, and one sample a multi-use model. `build.lights` is written from the lights the step lit by.

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
- A slot is planned (`LightPlan`): its points, each directional light's rays as a packet (its direction and the points
  it faces, by bit), each positioned light's rays made on the CPU one by one (`LightLampRays::Single`), so either
  backend traces the same rays. Its rays are traced by the backend, then each light's amounts made again and summed in
  the SDK's order, point by point.
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
the GPU the request names (`RayGpu`, the app's renderer choice), else the high-performance one. The ray model goes up
once to the tracer the lightmaps share (`RayGpuTracer`) as two geometries: the opaque faces, which the hardware takes
alone and which end a ray at the first; and the alpha faces, which the shader dims the light through
(`ray/gpu/shadow_rays.wgsl`). Faces passing light are left out. A packet's
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
over a hole, the actor moved only to another start level, the spawn GUID, the report (a connection left out, a wall
across the map and its strict refusal), and the refusals (no actor or two, a shared story id, a level of no entity, a
changer leading nowhere built or to a point its level lacks, two patrols of a name, an unknown start level). They take the
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
outside any game installation. The GPU tests, the tracer under each lighter's rules, the detail lighter's and the AI
map's covers against the CPU's, trace on Vulkan, and on D3D12 with the checkout's `.sdks/dxc`; a backend no GPU of the
host traces rays on is skipped.
