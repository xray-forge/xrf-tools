# xrf-level-compiler

Compiles a level from the files `level build` writes (`xrf-rawdata`'s `LevelBuildExporter`), step by step, into what the
game loads: the geometry with its lightmaps and collision, the details' light and the AI map.

| Step       | Reads                                                       | Runs today            |
| ---------- | ----------------------------------------------------------- | --------------------- |
| `geometry` | `build.prj`                                                 | the SDK's xrLC        |
| `details`  | `build.prj`, `build.details`, `build.cform`, `build.lights` | the SDK's xrDO_Light  |
| `ai`       | `build.prj`, `build.aimap`, `build.cform`                   | the SDK's xrAI (`-f`) |

`LevelCompiler::compile` takes a `LevelCompileRequest`: the level's directory, the steps (every one when none), the
SDK's `editors` directory, the game data whose `gamemtl.xr` and `shaders_xrlc.xr` the compilers read, the textures root,
and a working directory. It checks every step's inputs and program first, and a `.thm` under the textures root for
every texture `build.prj` names (each compiler loads them), so a missing file stops it before a long run,
then runs the steps in order in the working directory, under an `fsgame.ltx` it writes there, reporting each compiler's
last log line to an `xrf-job` `JobHandle` and killing the compiler when the job is cancelled. `xrf-cli level compile`
and the app's level compiler call it.

Each SDK compiler ends a run on a "Congratulation!" message box and waits on it, even once answered. xrLC and
xrDO_Light take `-silent` to skip it; xrAI has no such switch, so its box showing ends the step and the process is
stopped, its files already written (`sdk/level_sdk_dialog.rs`).

Each step runs the SDK 0.7 compiler until a compiler of this crate takes its place; the request, the steps and the
report stay as they are when one does. The spawn (xrAI `-s`) is not a step: it works on
every level of a game at once.

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

`DetailLighter::light` lights `build.details` as xrDO_Light does, and is not a step yet: the `details` step still runs
the SDK's. It takes a `DetailLightRequest` (the details, a tracer over `build.cform`, the shadow materials and
`build.lights`) and an `xrf-job` `JobHandle`, and gives a `DetailLighting`: the `level.details` to write and how many
slots it met each way.

- Every slot's corners whose densities are all zero are cleared; a slot planting nothing, or touching no face of the
  ray model in its box, is left unlit.
- A planted slot is lit from 15 by 15 points, each dropped onto the highest face under it, by every light of
  `build.lights` reaching the slot: the static lights' colour by the cosine and attenuation, the sun and the sky as
  plain amounts. Each light is scaled by the product of the transmittances its shadow ray crosses.
- A directional light's rays from a slot's points share its direction and trace as one packet. The amounts are then
  summed in the SDK's order, point by point, so the packets change no result, only the time.
- The sums are averaged, halved and stored as four bits each (`LevelDetailsLight::quantize`).

The slots run over the installed rayon pool, a progress unit each, and a cancelled job stops the lighting with
`XrfError::Cancelled`. Every shadow ray is a full query: the SDK's lighter first tests the face that last blocked each
light, from either side, so its result depends on the order slots run in and can come out darker; this one is the
same on any thread count. On the kept SDK runs of `mp_darkvalley` and `jupiter` it matches the SDK's slot for slot but
for one in about 20,000 to 90,000, each one step off: the SDK's cache, or a sum under the x87's precision.

## Develop

```bash
cargo test -p xrf-level-compiler
```

The tests run no compiler. A run of the SDK's compilers on a level is the CLI's or the app's, on copies of a level
outside any game installation.
