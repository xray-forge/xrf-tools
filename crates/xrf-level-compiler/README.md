# xrf-level-compiler

Compiles a level from the files `level build` writes (`xrf-rawdata`'s `LevelBuildExporter`), step by step, into what the
game loads: the geometry with its lightmaps and collision, the details' light and the AI map. Each step ports an SDK
compiler and runs in process.

| Step       | Ports           | Reads from the level's directory                                   | Writes                                                                                                                 |
| ---------- | --------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `geometry` | xrLC's geometry | `build.prj`                                                        | `level`, `level.geom`, `level.geomx`, `level.cform`, `build.cform`, `build.lights`, neutral lightmap and implicit maps |
| `light`    | xrLC's light    | as the geometry, in its pass                                       | the lit lightmap pages and implicit maps, in place of the neutral ones                                                 |
| `details`  | xrDO_Light      | `build.prj`, `build.details`, `build.cform`, `build.lights`        | `level.details`                                                                                                        |
| `ai`       | xrAI `-f`       | `build.aimap`; with full covers also `build.prj` and `build.cform` | `level.ai`                                                                                                             |

Every step but an AI map with no covers also reads the game data's `shaders_xrlc.xr` and the textures `build.prj` names.

The root spawn (xrAI `-s`) is not a step: it works on every level of a game at once, and `xrf-spawn-builder` builds it.

## Compiling a level

`LevelCompiler::compile` takes a `LevelCompileRequest`: the level's directory, the steps (every one when none, always
run in the table's order), the game data whose `gamemtl.xr`, `shaders_xrlc.xr` and `textures` the compilers read, the
textures roots searched in order before the game data's, whether the outputs may be overwritten, what traces the rays (`RayBackend`, DXC's library,
`RayGpu`), the AI map's covers (`AiMapCovers`) and the light's quality (`LightQuality`). `xrf-cli level compile` and
the app's level compiler call it.

Before anything runs it refuses the light step without the geometry step, a missing input of any step (one the geometry
step writes in the same compile aside), a missing compiler shader library, a texture `build.prj` names with no `.thm`
under the textures roots where a step reads textures, and an existing `level.details` or `level.ai` unless forced, so a
missing file stops it before a long run. Then it runs the steps in order, reporting to an `xrf-job` `JobHandle` and
stopping when the job is cancelled, each writing its outputs staged (`compile/`). Each step's report
(`LevelCompileStepReport`) gives its time and what ran it (`LevelCompileStepRun`): the geometry compiler with its
census, the light baker with its census and the GPU that traced, the detail lighter with the slots it met and the GPU
that traced, or the AI map compiler with its covers, GPU, identity and graph check.

## The geometry and light steps

The geometry step (`GeometryStep`, `geometry/`) compiles `build.prj` through xrLC's geometry phases (`GeometryLevel`):
the mesh loaded, welded and its long edges cut (`geometry/mesh/`), its normals, base coordinates and tangents
(`geometry/surface/`), the collision form and the compilers' ray model (`geometry/collision/`), the splits each visual
is made of (`geometry/split/`), the static visuals, progressive above draft quality (`geometry/visual/`), the multi-use
models' trees and LODs (`geometry/model/`), and the visual tree and sectors (`geometry/hierarchy/`).

The two collision forms place each multi-use face's corners as SDK 0.7's xrLC binary does, which sums
`transform_tiny`'s terms in its own order at each call (`GeometryPlacement`'s `RAY_MODEL_SUMS` and `COLLISION_SUMS`),
and the ray model leaves out a model face repeating another's split corners, as it does a static one. On `jupiter`
the ray model then holds the SDK's faces but for 115 and 83 of 3.09 million (one strip of static faces
triangulated otherwise), and `level.cform` all but 1,693 and 1,771 of 2.1 million; `mp_darkvalley`'s ray model is the SDK's.

The light step has no pass of its own. Where it runs, the geometry compile bakes the light within its pass
(`LightBaker::bake`, `light/`), since the lightmap pages decide which splits join into a visual; the bake's time and
census (`LightBake`) are reported as the light step's. Without it every lightmapped visual takes one pair of neutral
light (`LightDraft`). Either way the geometry step writes `build.lights` from `build.prj`'s light layers at the
request's quality (`LightLayers`), the lights the bake lights by.

`LightQuality` sets those lights and the samples: `Scene` takes `build.prj`'s own lights and sample counts; `Draft`
takes the sky and sun the Level Editor builds at quality 1 (`xrf_geometry::Sky`) in place of the `$hemi` and `$sun`
layers' directional lights, the lamps under them kept, lightmaps of 3 texels a metre and one sample, and one sample a
multi-use model.

### Vertex light

The vertex light (`light/vertex/`) is built as xrLC builds it, each term stored as `base_color`'s `fixed16<16>`
(`LightColor`):

- `LightHemisphere`: every welded vertex's sky term before the smoothing split, through the ray model of the welded
  mesh (`GeometryRayModel::build_welded`), halved and averaged with its neighbours five times;
- `LightVertices`: each split vertex's sky term, and a vertex-lit one's static light and sun, halved, then moved toward
  the brightest vertex at its place by its translucency;
- `LightModels`: each multi-use model's sky light in its own space, which its trees store, and each placement's light
  in the level, which gives its tree's scale and bias by a least-squares fit; samples are jittered from a seeded source
  where xrLC uses its random one.

All three trace on the CPU through xrLC's own ray model, `xrf-geometry`'s `XrlcTracer`: they are matched to xrLC vertex
for vertex, and xrLC's tree loses rays running exactly along a box's corner, which a tighter tree meets. `LightVertices`
also lights runs of vertices in order, remembering each light's last blocker from one to the next as xrLC's threads do
(`rayTrace`'s `L.tri`); the base hemisphere does not, as xrLC's eight threads there share that blocker between vertices
far apart. The lightmaps keep `BvhTracer`, which the GPU mirrors.

On the SDK's own vertex light of `l01_test` and `labx8`, `LightPoint`'s `Level` variant gives xrLC's bytes on 99.99% and
99.93% of vertices.

### Lightmaps

`light/lightmap/` cuts, sizes and lights charts as xrLC does and packs them its own way. `LightChart::cut` is
`xrPhase_UVmap`: per material split, the largest face not yet charted starts a chart, which takes, depth first through
the faces sharing a split vertex, every face within the smoothing angle and a degree of its normal; `LightChart::of` is
`OA_Export`, the projection along the faces' area-weighted normal and the size in texels from the density, between 1 and
510 a side. `GeometrySplits::cut` detaches each chart into a split, and a split the subdivision cuts is charted again a
half at a time; `GeometrySplits::merge` then joins only splits on one page.

`LightChartLighter` is `CDeflector::Light`: each texel the mean of its jitter samples that fall on a face, lit there
with `LightPoint` facing the corners' normals blended and joined with the face's, the face itself let through; then the
faces' edges walked an eighth of a texel at a time; each light halved and stored as `base_color` does
(`LightmapLayer`). Border rings fill the rest (`ApplyBorders`); a chart within its RMS of one colour becomes it, two
texels a side; else each side tries a half, a quarter or three quarters, kept where `LightmapResampler`'s Lanczos
shrink and cubic growth (`imf_Process`) hold, and the chart is lit again at the smaller size; then it takes its border.
On the CPU each light remembers the face that last stopped it on the chart (`LightOccluders`), as `rayTrace` does,
which one triangle test settles before a full query: the light is the same, ten times faster on `labx8`.

`LightChartMask` is what a lit chart takes on a page, its texels lit and two rings, kept as each row's span.
`LightmapAtlas::pack` takes the charts in `xrPhase_MergeLM`'s order and selection and places each on a 1024-texel page
at the first place by row it fits, upright before crosswise, over a bit a texel; a mask of the same spans starts from
where the last one fitted. `LightmapAtlas::compose` lays each chart's texels on its page, fills sixteen border rings
and packs `_1` (static light, sun in alpha) and `_2` (sun, hemisphere in alpha), which the step writes as BC3.

On the SDK's runs the charts hold the same triangles as xrLC's for 99.56 to 100% of them, the pages are as many, the
light, compared in world space, lies within the threshold two SDK bakes of a level differ by, and `jupiter`'s terrain
map within p99 6 steps of xrLC's texel for texel.

### Implicit maps

`ImplicitLighter` (`light/implicit/`) is `ImplicitLightingExec`, for a texture whose `.thm` says it is lit implicitly,
such as terrain's: a map the texture's size over its own coordinates, each texel the mean of every face under each of
its jitter samples, lit with the chart lighter's `LightPoint`; the rings then fill the map, the texture's alpha takes
the hemisphere and the map the static light and sun (`ImplicitMap`), which the step writes as `<texture>.dds` and
`<texture>_lm.dds` in the level's folder. As xrLC, it lights none at draft quality.

A terrain visual names its `<texture>_lm` whether or not the light step lit it. Without the light step, or at draft
quality, the geometry step writes neutral maps instead (`LightDraft::get_implicit_map`): the texture with a middle
hemisphere in alpha, and an `_lm` of no static light and full sun. A new level then draws its terrain lit by the
renderer alone, rather than with a missing map or an older level's. These maps need the texture's `.dds` under the
texture roots, as the lit ones do.

### On the GPU

`LightmapBackend` lights the charts and the implicit maps on the backend the request names, started from the chart
lighter's own ray scene (`RayScene`), so the GPU holds the ray model the lighter's faces are numbered in. On the GPU
(`LightmapGpu`) a chunk of charts' samples is planned a few hundred points a plan (`LightPlan`) with the lights reaching
them, every light's rays a packet the shader makes from the points (`RayLampMode::Packet`), traced in batches by the
shared `RayGpuTracer` under xrLC's rules (`RayRules::LEVEL`), finished on the pool; the charts that shrink are lit again
together, then each takes its border. A point's light is the CPU's but where the GPU's arithmetic parts from it in a
last bit, which a chart's compression can carry across it: on the proof levels the pages lie within p90 3, p99 12 steps
of the CPU's, and as close to xrLC's as the CPU's are.

## The detail lighter

`DetailLighter::light` (`detail/`) lights `build.details` as xrDO_Light does. It takes a `DetailLightRequest` (the
details, the ray scene over `build.cform` with its shadow materials, `build.lights`, and what traces, `RayTracing`) and
an `xrf-job` `JobHandle`, and gives a `DetailLighting`: the `level.details` to write, how many slots it met each way
(`DetailSlotCounts`), and the GPU that traced. The details step (`DetailStep`) runs it on a level directory.

- Every slot's corners whose densities are all zero are cleared; a slot planting nothing, or touching no face of the
  ray model in its box, is left unlit.
- A planted slot is lit from 15 by 15 points, each dropped onto the highest face under it, by every light of
  `build.lights` reaching the slot: the static lights' colour by the cosine and attenuation, the sun and the sky as
  plain amounts. Each light is scaled by the product of the transmittances its shadow ray crosses.
- A slot is planned (`LightPlan`): its points, each directional light's rays as a packet (its direction and the points
  it faces, by bit), each positioned light's rays made on the CPU one by one (`RayLampMode::Single`), so either backend
  traces the same rays. Its rays are traced by the backend, then each light's amounts made again and summed in the
  SDK's order, point by point.
- The sums are averaged, halved and stored as four bits each (`LevelDetailsLight::quantize`).

Every shadow ray is a full query: the SDK's lighter first tests the face that last blocked each light, from either side,
so its result depends on the order slots run in and can come out darker; this one is the same on any thread count. The
slots run a progress unit each, and a cancelled job stops the lighting with `XrfError::Cancelled`.

On the kept SDK runs of `mp_darkvalley` and `jupiter` the CPU matches the SDK's slot for slot but for one in about
20,000 to 90,000, each one step off: the SDK's cache, or a sum under the x87's precision. On the GPU (`detail/gpu/`)
slots are planned on the pool a batch at a time and their rays traced together; a packet's rays are made on the GPU
from its slot's origins, a workgroup a packet, so a slot sends about 20 KB rather than its rays. It matches the CPU on
every `mp_darkvalley` slot and all but 9 of `jupiter`'s 345,801, each one step off.

## The AI map compiler

`AiMapCompiler::compile` (`ai/`) compiles `build.aimap` into `level.ai` as SDK 0.7's `xrAI -f` does. It takes an
`AiMapRequest` (the map, the covers, the ray scene they are traced against, what traces, and the `level.ai` it
replaces, if any) and an `xrf-job` `JobHandle`, and gives an `AiMapCompilation`: the `AiMap`, header and nodes, where
its identity came from (`AiMapGuid`), and the GPU that traced. The AI step (`AiStep`) runs it on a level directory:
the replaced `level.ai` read for its identity, the compiled map checked as a graph (`xrf-level`'s
`LevelAiGraphVerifier`) and refused with a node linked to itself before it is written.

- Each node is loaded as xrAI loads it: its cell through the packed grid over the map's box and back (`AiGrid`), its
  plane normal decompressed and normalised.
- `AiMapCovers::None` stores every cover as 0, as xrAI's `-draft` does. `Full` needs a `RayScene` over `build.cform`
  and its shadow materials, and computes the covers as xrAI's `xrCover` does (`ai/cover/`), in single precision and
  the engine's order:
  - the corner nodes, where a link is missing beside a missing link or a neighbour reaching round (a straight edge's
    nodes are not ones), each trace a ray toward every node linked to them within 30 m, from 1.5 and 0.6 m up; each
    of eight sectors keeps the share its rays let through, and each of four covers averages four sectors;
  - every other node mixes the corner nodes not open both ways that the engine's quadtree lists within 30 m and its
    straight walk reaches, weighted by one over distance; with none, it is open.

  Every ray is a full query, never the SDK's remembered blocker, so the covers are the same on any thread count.
  Corner nodes and other nodes run over the installed rayon pool, a progress unit each; a cancelled job stops with
  `XrfError::Cancelled`.

- The corner rays are traced on the backend the request names, by xrAI's rules (`RayRules::COVERS`, the detail
  lighter's). On the GPU (`ai/cover/gpu/`) nodes are flooded and their rays made on the pool a batch at a time while
  the GPU traces the batch before, one ray at a time; the sums stay on the CPU.
- The save: the box around every node, its height grown by `EPS_L`; each node's cell, height, plane and covers
  compressed over it; a stable sort by packed cell with the links renumbered; the light 15, which no loader reads.
- The identity is the replaced map's while its graph stands (as many nodes, the same cell size, each at the same cell
  with the same links), so the existing root spawn still loads the level; otherwise the first 16 bytes of the SHA-256
  of the map with a nil identity, so the same input gives the same file. xrAI draws a random one every run.

On the kept SDK runs of `labx8` and `jupiter` a draft compile is the SDK's `-draft` file byte for byte but for the
identity and each node's light. A full compile is the full run's byte for byte on `labx8`; on `jupiter` every cover
value is within one step, 99.97% exact, the rest single rays that differ and the mix carrying them, on either backend;
the GPU compiles `jupiter` in about 10 s, the CPU in 35.

## The ray layer

`ray/` is what the light baker, the detail lighter and the AI map's covers trace their rays through, one layer for all
three.

- `RayScene` is a ray model in a `BvhTracer` with its `ShadowMaterials`. The details and AI steps read it from
  `build.cform` (`RayModel`); the light baker builds its own from the ray model the geometry compile just made, for
  the lightmaps and implicit maps (its vertex passes take an `XrlcTracer` over the same faces).
- `RayRules` says how a ray meets the faces, one source for the CPU and the GPU: `DETAILS` and `COVERS` stop at front
  faces and let `1 - alpha` through an alpha texel; `LEVEL` stops at either side, lets `1 - alpha²` through, passes
  the sample's own face, and has the GPU test each face it meets again by the CPU's ray test.
- A lighter's rays are planned (`LightPlan`) as groups (`RayGroup`): a directional light's rays as a packet; a light
  at a position's as a packet whose rays are made where they are traced (`RayLampMode::Packet`, the lightmaps' on the
  GPU), or as rays made on the CPU one by one (`RayLampMode::Single`, the detail lighter's), which both backends then
  trace alike to the last bit.
- `RayTracing`, from `LevelCompileRequest::get_tracing`, names the backend, DXC's library and the GPU. Each compiler
  starts its device through `RayDevice`, which holds the one `Auto` fallback to the CPU and the refusal where the
  build has no GPU backend.

| Backend (`RayBackend`) | Traces                                                                     |
| ---------------------- | -------------------------------------------------------------------------- |
| `Auto`, the default    | the GPU where this build and the machine can, the CPU otherwise            |
| `Gpu`                  | with hardware ray queries; refused where the build or the machine cannot   |
| `Cpu`                  | on the installed rayon pool; the reference the SDK's runs are matched with |

The GPU backend is the `gpu` feature, off by default, so a build without it links no graphics code; the CLI and the
app turn it on. `RayDevice` starts an `xrf-renderer-core` `RayQueryDevice`: D3D12 where DXC's library is given, else
Vulkan; on the GPU the request names (`RayGpu`, the app's renderer choice), else the high-performance one. The ray
model goes up once to the shared tracer (`RayGpuTracer`, `ray/gpu/`) as two geometries: the opaque faces, which the
hardware takes alone and which end a ray at the first; and the alpha faces, which the shader dims the light through
(`ray/gpu/shadow_rays.wgsl`). Faces passing light are left out.

### Shadow materials

`ShadowMaterials::load` (`shadow/`) reads how each face of xrLC's ray model (`build.cform`) meets a light ray, as the
SDK's light compilers read it. It takes a `ShadowMaterialsRequest`: `build.prj`, the ray model's faces,
`shaders_xrlc.xr` and the textures roots, searched in order: the roots the request names, then the game data's own
`textures` (`LevelCompileRequest::get_textures_roots`).

| Class    | When                                                                                             |
| -------- | ------------------------------------------------------------------------------------------------ |
| `Passes` | the material's compiler shader casts no shadow                                                   |
| `Alpha`  | the texture's `.thm` gives an alpha channel, and it is neither a `lod\` texture nor `level_lods` |
| `Opaque` | anything else                                                                                    |

An alpha face's image is decoded with `xrf-dds`' `decode_sdk_alpha`, and only for alpha textures a face names.
`get_opacity` answers what a tracer hit (`xrf-geometry`'s `TraceHit`, faces by index) stops: 0 for a face that passes
the ray, 1 for an opaque face, and for an alpha face the alpha at the nearest texel of the hit's texture coordinates,
wrapped. What passes is the tracing's `RayRules`'.

Loading fails where the SDK's compilers stop: a texture with no `.thm`, a compiler shader the library lacks, a face of
a material with no compiler shader, or an alpha image that is missing or not the size its `.thm` gives.

### `LightPoint`

`LightPoint::evaluate` (`light/point/`) gathers the light a point with a normal receives from the lights of
`build.lights`, by role (`LightSources`, `LightRoles`): static lights' colour by the cosine, the sun and the sky as
amounts, lights at a position fading with distance, each scaled by what its shadow ray lets through the ray model. Its
two variants (`LightPointVariant`) are the two SDK compilers' copies, which differ in a static point light and in their
shadow rays' rules:

| Variant   | Static point light                                | Faces that shadow | An alpha texel lets through |
| --------- | ------------------------------------------------- | ----------------- | --------------------------- |
| `Details` | over its attenuation (xrDO_Light)                 | front only        | `1 - alpha`                 |
| `Level`   | its attenuation's inverse less its falloff (xrLC) | either side       | `1 - alpha²`                |

The detail lighter plans its rays through `LightPoint::get_ray`; the light baker's lighters share it.

## Develop

```bash
cargo test -p xrf-level-compiler
cargo test -p xrf-level-compiler --features gpu
```

The tests run no SDK compiler; comparisons with the SDK's own runs read output those runs left. The AI map tests
compile a generated four-node map: the sort and renumbering, the header, three planes whose round trip xrAI's own run
gave, the identity rule, and the refusals. The cover tests take a grid: the sectors, which nodes are corners, the walk
stopped by a hole, the quadtree's reach, and a wall whose front blocks the corners' rays and whose mix reaches the
other nodes.

The GPU tests, the tracer under each lighter's rules, the detail lighter's and the AI map's covers against the CPU's,
trace on Vulkan, and on D3D12 with the checkout's `.sdks/dxc`; a backend no GPU of the host traces rays on is skipped.
