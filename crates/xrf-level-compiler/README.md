# xrf-level-compiler

Compiles a level from the files `level build` writes (`xrf-rawdata`'s `LevelBuildExporter`), step by step, into what the
game loads: the geometry with its lightmaps and collision, the details' light and the AI map.

| Step       | Reads                                                       | Runs today                            |
| ---------- | ----------------------------------------------------------- | ------------------------------------- |
| `geometry` | `build.prj`                                                 | the SDK's xrLC                        |
| `details`  | `build.prj`, `build.details`, `build.cform`, `build.lights` | this crate's lighter (`DetailLighter`) |
| `ai`       | `build.prj`, `build.aimap`, `build.cform`                   | the SDK's xrAI (`-f`)                 |

`LevelCompiler::compile` takes a `LevelCompileRequest`: the level's directory, the steps (every one when none), the
SDK's `editors` directory for the steps the SDK runs (`LevelCompileStep::is_sdk`), the game data whose `gamemtl.xr` and
`shaders_xrlc.xr` the compilers read, the textures roots (the SDK's steps take exactly one), a working directory for the
SDK's compilers, whether `level.details` may be overwritten, the details' backend and DXC's library for it. It checks
every step's inputs and program first, and a `.thm` under the textures roots for every texture `build.prj` names (each
compiler loads them), so a missing file stops it before a long run. Then it runs the steps in order: the SDK's in the
working directory, under an `fsgame.ltx` it writes there, reporting each compiler's last log line to an `xrf-job`
`JobHandle` and killing it when the job is cancelled; the details in process (`DetailStep`), writing `level.details`
staged. Each step's report says what ran it (`LevelCompileStepRun`): an SDK program and its log, or the lighter, the
slots it met and the GPU that traced. `xrf-cli level compile` and the app's level compiler call it.

Each SDK compiler ends a run on a "Congratulation!" message box and waits on it, even once answered. xrLC takes
`-silent` to skip it; xrAI has no such switch, so its box showing ends the step and the process is stopped, its files
already written (`sdk/level_sdk_dialog.rs`).

The SDK 0.7 compilers run the steps no compiler of this crate does yet. The spawn (xrAI `-s`) is not a step: it works on
every level of a game at once.

## The AI map compiler

`AiMapCompiler::compile` compiles `build.aimap` into `level.ai` as SDK 0.7's `xrAI -f` does. It takes an `AiMapRequest`
(the map, the covers, the ray model they are traced against, and the `level.ai` it replaces, if any) and an `xrf-job`
`JobHandle`, and gives an `AiMapCompilation`: the `AiMap`, header and nodes, and where its identity came from
(`AiMapGuid`). It is not yet the `ai` step.

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
- The save: the box around every node, its height grown by `EPS_L`; each node's cell, height, plane and covers
  compressed over it; a stable sort by packed cell with the links renumbered; the light 15, which no loader reads.
- The identity is the replaced map's while its graph stands (as many nodes, the same cell size, each at the same cell
  with the same links), so the existing spawn still loads the level; otherwise the first 16 bytes of the SHA-256 of the
  map with a nil identity, so the same input gives the same file. xrAI draws a random one every run.

On the kept SDK runs of `labx8` and `jupiter` a draft compile is the SDK's `-draft` file byte for byte but for the
identity and each node's light. A full compile is the full run's byte for byte on `labx8`; on `jupiter` every cover
value is within one step, 99.97% exact, the rest single rays that differ and the mix carrying them.

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

| Backend (`DetailLightBackend`) | Traces                                                                                  |
| ------------------------------ | --------------------------------------------------------------------------------------- |
| `Auto`, the default            | the GPU where this build and the machine can, the CPU otherwise                          |
| `Gpu`                          | slots planned on the pool a batch at a time, their rays made and traced with ray queries  |
| `Cpu`                          | each slot whole on a thread of the installed rayon pool, a packet at a time; the reference |

The GPU backend is the `gpu` feature, off by default, so a build without it links no graphics code; the CLI and the
app turn it on. It starts an `xrf-renderer-core` `RayQueryDevice`: D3D12 where DXC's library is given, else Vulkan; on
the GPU the request names (`DetailGpu`, the app's renderer choice), else the high-performance one. The ray model goes up once as two
geometries: the opaque faces, which the hardware takes alone and which end a ray at the first; and the alpha faces, which
the shader dims the light through (`detail/gpu/detail_shadow_rays.wgsl`). Faces passing light are left out. A packet's
rays are made on the GPU from its slot's origins, a workgroup a packet, so a slot sends about 20 KB rather than its
rays. On the kept runs it matches the CPU on every `mp_darkvalley` slot and all but 9 of `jupiter`'s 345,801, each one
step off.

The slots run a progress unit each, and a cancelled job stops the lighting with `XrfError::Cancelled`.

The AI map tests compile a generated four-node map: the sort and renumbering, the header, three planes whose round
trip xrAI's own run gave, the identity rule, and the refusals. The cover tests take a grid: the sectors, which nodes are
corners, the walk stopped by a hole, the quadtree's reach, and a wall whose front blocks the corners' rays and whose
mix reaches the other nodes.

## Develop

```bash
cargo test -p xrf-level-compiler
cargo test -p xrf-level-compiler --features gpu
```

The tests run no SDK compiler. A run of the SDK's compilers on a level is the CLI's or the app's, on copies of a level
outside any game installation. The GPU tests trace on Vulkan, and on D3D12 with the checkout's `.sdks/dxc`; a backend
no GPU of the host traces rays on is skipped.
