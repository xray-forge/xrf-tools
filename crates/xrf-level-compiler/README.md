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

## Develop

```bash
cargo test -p xrf-level-compiler
```

The tests run no compiler. A run of the SDK's compilers on a level is the CLI's or the app's, on copies of a level
outside any game installation.
