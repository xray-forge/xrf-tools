# xrf-level-compiler

Compiles a level from the files `level build` writes (`xrf-rawdata`'s `LevelBuildExporter`) into what the game loads:
the geometry with its lightmaps and collision, the details' light and the AI map. Each step ports an SDK compiler and
runs in process. `xrf-cli level compile` and the app's level compiler call it.

| Step       | Ports           | Reads from the level's directory                                   | Writes                                                                                                                 |
| ---------- | --------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `geometry` | xrLC's geometry | `build.prj`                                                        | `level`, `level.geom`, `level.geomx`, `level.cform`, `build.cform`, `build.lights`, neutral lightmap and implicit maps |
| `light`    | xrLC's light    | as the geometry, in its pass                                       | the lit lightmap pages and implicit maps, in place of the neutral ones                                                 |
| `details`  | xrDO_Light      | `build.prj`, `build.details`, `build.cform`, `build.lights`        | `level.details`                                                                                                        |
| `ai`       | xrAI `-f`       | `build.aimap`; with full covers also `build.prj` and `build.cform` | `level.ai`                                                                                                             |

Every step but an AI map with no covers also reads the game data's `shaders_xrlc.xr` and the textures `build.prj` names.
The root spawn (xrAI `-s`) works on every level of a game at once, so it is not a step: `xrf-spawn-builder` builds it.

## Compile a level

`LevelCompiler::compile` takes a `LevelCompileRequest` and an `xrf-job` `JobHandle`:

- `level`: the level's directory;
- `steps`: the steps to run, always in the table's order; every step when empty;
- `gamedata` and `textures`: the game data holding `gamemtl.xr` and `shaders_xrlc.xr`, and the textures roots searched
  before the game data's own `textures`;
- `is_forced`: whether existing outputs may be replaced;
- `backend`, `dxc` and `gpu`: what traces the rays (`RayBackend`, DXC's library, `RayGpu`);
- `ai_covers` (`AiMapCovers`) and `light_quality` (`LightQuality`).

Before any step runs, it refuses:

- the light step without the geometry step;
- a missing input of any step, except one the geometry step writes in the same compile;
- a missing `shaders_xrlc.xr` where a step reads it;
- a texture `build.prj` names with no `.thm` under the textures roots, where a step reads textures;
- an output already there, unless forced.

Each step writes its outputs staged, so a failed write leaves the old file. A cancelled job stops the compile with
`XrfError::Cancelled`. The `LevelCompileReport` holds one `LevelCompileStepReport` per step: its time and what ran it
(`LevelCompileStepRun`), with that compiler's census and the GPU that traced, if any.

## Geometry and light

The geometry step (`geometry/`) runs xrLC's geometry phases on `build.prj` (`GeometryLevel`): the mesh welded and its
long edges cut, normals and tangents, the collision form and the compilers' ray model, the splits, the static visuals
(progressive above draft quality), the multi-use models' trees and LODs, and the visual tree and sectors.

The collision forms place each multi-use face's corners as the SDK 0.7 xrLC binary does, summing `transform_tiny`'s terms
in its order (`GeometryPlacement`'s `RAY_MODEL_SUMS` and `COLLISION_SUMS`). On `jupiter` the ray model holds the SDK's
faces but for 115 and 83 of 3.09 million, and `level.cform` all but 1,693 and 1,771 of 2.1 million; on `mp_darkvalley`
the ray model is the SDK's.

The light step has no pass of its own: the lightmap pages decide which splits join into a visual, so the geometry pass
bakes the light where the step runs (`LightBaker::bake`, `light/`) and reports the bake as the light step. Without it
every lightmapped visual takes neutral light (`LightDraft`). Either way the geometry step writes `build.lights` from
`build.prj`'s light layers (`LightLayers`).

`LightQuality::Scene` keeps `build.prj`'s lights and sample counts. `Draft` replaces the `$hemi` and `$sun` layers'
lights with the sky and sun the Level Editor builds at quality 1 (`xrf_geometry::Sky`), keeps the lamps, and lights
lightmaps at 3 texels a metre with one sample, and multi-use models with one sample.

### Vertex light

`light/vertex/` lights as xrLC does, each term stored as `base_color`'s `fixed16<16>` (`LightColor`):

- `LightHemisphere`: each welded vertex's sky term through the welded mesh's ray model, smoothed over its neighbours;
- `LightVertices`: each split vertex's sky term, and a vertex-lit one's static light and sun;
- `LightModels`: each multi-use model's sky light in its own space, and each placement's tree scale and bias by a
  least-squares fit; samples are jittered from a seeded source where xrLC uses its random one.

These trace on the CPU through xrLC's own ray model (`xrf-geometry`'s `XrlcTracer`), so they lose the rays xrLC's tree
loses. `LightVertices` also keeps each light's last blocker from one vertex to the next, as xrLC's threads do
(`rayTrace`'s `L.tri`). On the SDK's vertex light of `l01_test` and `labx8` they give xrLC's bytes on 99.99% and 99.93%
of vertices.

### Lightmaps

`light/lightmap/` cuts, sizes and lights charts as xrLC does and packs them its own way:

| Type                  | xrLC counterpart                                                 |
| --------------------- | ---------------------------------------------------------------- |
| `LightChart::cut`     | `xrPhase_UVmap`                                                  |
| `LightChart::of`      | `OA_Export`, 1 to 510 texels a side                              |
| `LightChartLighter`   | `CDeflector::Light`, with `ApplyBorders`                         |
| `LightmapResampler`   | `imf_Process`, which decides whether a chart may shrink          |
| `LightmapAtlas::pack` | `xrPhase_MergeLM`'s chart order and selection; its own placement |

Pages are 1024 texels a side, written as BC3: `_1` holds the static light with the sun in alpha, `_2` the sun with the
hemisphere in alpha. On the CPU each light remembers the face that last stopped it on a chart (`LightOccluders`), as
`rayTrace` does, which lights `labx8` ten times faster with the same result.

On the SDK's runs the charts hold the same triangles as xrLC's for 99.56 to 100% of them, the page count matches, the
light lies within the threshold two SDK bakes of a level differ by, and `jupiter`'s terrain map is within p99 6 steps of
xrLC's.

### Implicit maps

`ImplicitLighter` (`light/implicit/`) is `ImplicitLightingExec`, for a texture whose `.thm` marks it lit implicitly, such
as terrain's. It writes `<texture>.dds`, the hemisphere in alpha, and `<texture>_lm.dds`, the static light and sun, in
the level's directory. Like xrLC it lights none at draft quality.

Without the light step, or at draft quality, the geometry step writes neutral maps (`LightDraft::get_implicit_map`): a
middle hemisphere, no static light and full sun, so a new level draws its terrain lit by the renderer alone. Both need
the texture's `.dds` under the textures roots.

### On the GPU

`LightmapBackend` lights charts and implicit maps on the request's backend from the chart lighter's own ray scene. On
the GPU (`LightmapGpu`) a chart's points are planned (`LightPlan`), each light's rays made in the shader from the points
(`RayLampMode::Packet`) and traced by the shared `RayGpuTracer` under `RayRules::LEVEL`. The GPU's arithmetic can differ
from the CPU's in a last bit, which compression can carry: on the proof levels the pages lie within p90 3, p99 12 steps
of the CPU's, and as close to xrLC's as the CPU's are.

## Detail lighter

`DetailLighter::light` (`detail/`) lights `build.details` as xrDO_Light does, from a `DetailLightRequest`, and gives a
`DetailLighting`: the `level.details` to write, the slots it met each way (`DetailSlotCounts`) and the GPU that traced.

- A slot's corners whose densities are all zero are cleared. A slot planting nothing, or touching no face of the ray
  model, is left unlit.
- A planted slot is lit from 15 by 15 points, each dropped onto the highest face under it, by every light of
  `build.lights` reaching the slot, each scaled by the transmittance its shadow ray crosses.
- The sums are averaged, halved and stored as four bits each (`xrf-level`'s `LevelDetailsLight::quantize`).

Every shadow ray is a full query: the SDK's lighter first retests the face that last blocked each light, so its result
depends on slot order and can come out darker. This one is the same on any thread count.

On the kept SDK runs of `mp_darkvalley` and `jupiter` the CPU matches the SDK slot for slot but for one in 20,000 to
90,000, each one step off. The GPU (`detail/gpu/`) makes a packet's rays from its slot's origins, about 20 KB a slot,
and matches the CPU on every `mp_darkvalley` slot and all but 9 of `jupiter`'s 345,801, each one step off.

## AI map compiler

`AiMapCompiler::compile` (`ai/`) compiles `build.aimap` into `level.ai` as SDK 0.7's `xrAI -f` does, from an
`AiMapRequest`, and gives an `AiMapCompilation`: the `AiMap`, where its identity came from (`AiMapGuid`) and the GPU that
traced. The AI step checks the map as a graph (`xrf-level`'s `LevelAiGraphVerifier`) and refuses a node linked to itself
before writing it.

- Nodes load as xrAI loads them, each cell packed and unpacked through `xrf-level`'s `LevelAiGrid`.
- `AiMapCovers::None` stores every cover as 0, as xrAI's `-draft` does. `Full` computes the covers as `xrCover` does
  (`ai/cover/`), in single precision and the engine's order: corner nodes trace rays toward linked nodes within 30 m,
  from 1.5 and 0.6 m up, into eight sectors and four covers; every other node mixes the corner nodes within 30 m that
  its straight walk reaches, weighted by one over distance.
- Every ray is a full query, so the covers are the same on any thread count. On the GPU (`ai/cover/gpu/`) the GPU
  traces the corner rays under `RayRules::COVERS`; the sums stay on the CPU.
- The identity is the replaced map's while its graph stands (as many nodes, the same cell size, each at the same cell
  with the same links), so the existing root spawn still loads the level. Otherwise it is the first 16 bytes of the
  SHA-256 of the map with a nil identity, so the same input gives the same file; xrAI draws a random one.

On the kept SDK runs a draft compile is the SDK's `-draft` file byte for byte on `labx8` and `jupiter`, the identity and
each node's unused light aside. A full compile is the SDK's byte for byte on `labx8`; on `jupiter` every cover is within
one step and 99.97% exact on either backend, and the GPU compiles it in about 10 s against the CPU's 35.

## Ray layer

`ray/` is the one layer the light baker, the detail lighter and the AI map's covers trace through.

- `RayScene` is a ray model in a `BvhTracer` with its `ShadowMaterials`. The details and AI steps read it from
  `build.cform` (`RayModel`); the light baker builds its own from the ray model the geometry pass just made.
- `RayRules` states how a ray meets faces, one source for the CPU and the GPU: `DETAILS` and `COVERS` stop at front
  faces and let `1 - alpha` through an alpha texel; `LEVEL` stops at either side, lets `1 - alpha²` through and passes
  the sample's own face.
- A lighter plans its rays (`LightPlan`) as groups (`RayGroup`): a directional light's rays as a packet; a positioned
  light's as a packet made where traced (`RayLampMode::Packet`, the lightmaps on the GPU) or as rays made on the CPU
  (`RayLampMode::Single`, the detail lighter), which both backends then trace alike.
- `RayDevice` starts each compiler's device from `LevelCompileRequest::get_tracing`, falling back to the CPU under
  `Auto` and refusing `Gpu` where the build has no GPU backend.

| Backend (`RayBackend`) | Traces                                                                     |
| ---------------------- | -------------------------------------------------------------------------- |
| `Auto`, the default    | the GPU where this build and the machine can, the CPU otherwise            |
| `Gpu`                  | with hardware ray queries; refused where the build or the machine cannot   |
| `Cpu`                  | on the installed rayon pool; the reference the SDK's runs are matched with |

The GPU backend is the `gpu` feature, off by default, so a build without it links no graphics code; the CLI and the app
turn it on. It starts an `xrf-renderer-core` `RayQueryDevice`: D3D12 where DXC's library is given, else Vulkan, on the
requested GPU or the high-performance one. The ray model is uploaded once as two geometries: opaque faces, which end a
ray, and alpha faces, which the shader dims the light through (`ray/gpu/shadow_rays.wgsl`). Faces passing light are left
out.

### Shadow materials

`ShadowMaterials::load` (`shadow/`) classes each face of `build.cform` as the SDK's light compilers do, from a
`ShadowMaterialsRequest`: `build.prj`, the faces, `shaders_xrlc.xr` and the textures roots
(`LevelCompileRequest::get_textures_roots`).

| Class    | When                                                                                             |
| -------- | ------------------------------------------------------------------------------------------------ |
| `Passes` | the material's compiler shader casts no shadow                                                   |
| `Alpha`  | the texture's `.thm` gives an alpha channel, and it is neither a `lod\` texture nor `level_lods` |
| `Opaque` | anything else                                                                                    |

`get_opacity` answers what a hit (`xrf-geometry`'s `TraceHit`) stops: 0 for a passing face, 1 for an opaque one, and
for an alpha face the alpha at the nearest wrapped texel, decoded with `xrf-dds`' `decode_sdk_alpha`.

Loading fails where the SDK's compilers stop: a texture with no `.thm`, a compiler shader the library lacks, a face of a
material with no compiler shader, or an alpha image that is missing or not the size its `.thm` gives.

### Light at a point

`LightPoint::evaluate` (`light/point/`) gathers the light a point with a normal receives from `build.lights`: static
lights' colour by the cosine, the sun and the sky as amounts, positioned lights fading with distance, each scaled by
what its shadow ray lets through. Its variants (`LightPointVariant`) are the two SDK compilers' copies:

| Variant   | Static point light                                | Faces that shadow | An alpha texel lets through |
| --------- | ------------------------------------------------- | ----------------- | --------------------------- |
| `Details` | over its attenuation (xrDO_Light)                 | front only        | `1 - alpha`                 |
| `Level`   | its attenuation's inverse less its falloff (xrLC) | either side       | `1 - alpha²`                |

## Checks

```bash
cargo test -p xrf-level-compiler
cargo test -p xrf-level-compiler --features gpu
```

The tests run no SDK compiler; comparisons with SDK runs read the output those runs left. The GPU tests compare each
GPU path with the CPU's on Vulkan, and on D3D12 with the checkout's `.sdks/dxc`; a backend no host GPU traces rays on is
skipped.
