# xrf-shaders

Reads X-Ray shaders in the three forms the game ships:

| Form               | Type                             | Holds                                                                   |
| ------------------ | -------------------------------- | ----------------------------------------------------------------------- |
| `shaders.xr`       | `ShaderLibraryFile`              | The compiled blender library                                            |
| `shaders_xrlc.xr`  | `ShaderCompilerFile`             | The level compiler's per-shader settings                                |
| `shaders\` sources | `XRayShaderScript`, `XRayShader` | `.s` Lua scripts, and the HLSL or GLSL sources they name, with includes |

- [xrf-material](../xrf-material/README.md) resolves how a surface looks from the library and the scripts.
- `xrf-gamedata` verifies the library, the scripts and the sources.
- The desktop app describes both archived tables.

## The blender library

A shader name that a mesh, level surface or config declares, such as `models\model_aref`, resolves to one compiled
blender in `shaders.xr`.

`ShaderLibraryFile` reads the library from a path, an open file, bytes (an archived library has no file to open), or a
chunk reader, always in X-Ray's byte order.

- `find_blender` and `contains_blender` look a blender up by its exact name.
- `blenders` iterates in no particular order.

### Reading rules

The engine and the SDK open blender chunks by id, from 0 upwards, until one is missing, whatever order the file stores
them in. Reading does the same, and refuses:

- a chunk that walk never reaches (past a gap, or a repeated id), since the game would resolve no surface to it;
- a library without its blenders chunk (id 2);
- a name defined twice;
- a blender chunk not consumed whole;
- a property type the engine never writes (11 and above, `XrfError::NotImplemented`), since guessing its size would
  misread every property after it.

### Blenders

A `ShaderBlender` holds the description `CBlender_DESC` lays out (its class, name, the computer and time the SDK saved
it at, and the class's format version), then its properties in the order the class wrote them.

- `class` is a `ShaderBlenderClass`: the engine's eight-character tag packed into a `u64`. Constants name the known
  ones (`MODEL`, `DEFAULT_AREF`, `TREE`, …), and `tag()` prints one without its padding.
- A property is a name and a `ShaderBlenderPropertyValue` of one of eleven kinds.
- Markers start groups: `group(marker)` returns the properties up to the next marker, and `texture(marker, name)` one
  texture in it.
- `integer`, `boolean` and `token` read a setting by name; `None` when the class does not write it or it is another
  kind.
- `base_texture(textures)` returns the texture the base slot resolves to for a surface compiled against that texture
  list: exactly `$base0` to `$base7` index the list, `$null` or an empty name is no texture, and anything else
  (`$base8` included) names itself.

Writing:

- `ShaderBlender` and `ShaderBlenderProperty` implement `xrf_chunk::ChunkReadWrite` and write back byte for byte. The
  library itself has no writer.
- Fixed-width text fields are Windows-1251 and null-terminated. A value too long for its field is refused on write.

## The compiler table

`ShaderCompilerFile` reads `shaders_xrlc.xr`, the per-shader settings the level compiler bakes with. It is a plain array
of 144-byte `ShaderCompilerShader` records: a 128-byte name, flags, vertex translucency, vertex ambient, and lightmap
density in texels per unit.

- Its readers take the byte order as a type parameter; pass `XRayByteOrder`.
- A size that is not a whole number of records is refused, as the engine asserts.
- Bytes the editor left in a name field after the terminator are kept (`name_trailing`), so `write` reproduces the file
  byte for byte. A name changed without them is refused.
- `find_shader` matches a name ignoring ASCII case, as the compiler does.
- `get_named_flags` names the set bits from `ShaderCompilerShader::FLAGS`.
- Both types serialize with serde, in camelCase.

## Shader scripts

A `.s` script is Lua the renderer runs to build a shader's passes. `XRayShaderScript::parse` reads one without running
it:

```rust
use xrf_shaders::{XRayShaderPass, XRayShaderScript};

fn main() -> xrf_error::XrfResult {
  let script = XRayShaderScript::parse(
    "shaders/r3/example.s",
    "function normal(shader) shader:begin(\"vertex\", \"pixel\") : zb(true, false) end",
  )?;
  let pass = script.pass_of(XRayShaderPass::BASE_FUNCTION).expect("a base pass");

  assert_eq!(pass.vertex_shader(), "vertex");
  assert_eq!(pass.pixel_shader(), "pixel");
  assert!(pass.state().is_depth_tested && !pass.state().is_depth_written);

  Ok(())
}
```

### Passes

A pass is a `shader:begin` call with two literal string arguments, or three where DX10 and later put a geometry program
between the vertex and pixel programs.

- Computed names are not evaluated.
- Calls in comments, including a commented-out `--[[ ]]` draft, produce no pass.
- A Lua syntax error is `XrfError::Verify`.

The function a pass is declared in decides when the engine compiles it:

| Constant                 | Function    | Element                                                       |
| ------------------------ | ----------- | ------------------------------------------------------------- |
| `BASE_FUNCTION`          | `normal`    | A surface's base element                                      |
| `DETAILED_BASE_FUNCTION` | `normal_hq` | Replaces `normal` where the base texture has a detail texture |
| `SPECIAL_FUNCTION`       | `l_special` | The distortion or emissive element                            |
| `SHADOW_FUNCTION`        | `l_point`   | The deferred renderers' shadow element                        |

`pass_of(function)` returns that function's first pass, or `None`.

### State and samplers

`state()` is the render state chained onto `begin`: `blend`, `aref`, `zb`, `wmark`, `distort` and `emissive`. A pass
that chains nothing tests and writes depth and does not blend.

`samplers()` are the `shader:sampler(name):texture(texture)` and DX10 `shader:dx10texture(name, texture)` calls after
`begin` and before the function's next `begin`, in source order. `dx10sampler` binds no texture and is skipped.

Each texture is an `XRayShaderSamplerTexture`:

| Variant      | Meaning                                                                                                            |
| ------------ | ------------------------------------------------------------------------------------------------------------------ |
| `Named`      | A literal, or a local bound to one. May be an engine target such as `$user$sky0`; `file()` returns only real files |
| `Parameter`  | An argument the renderer passes, such as `t_base`                                                                  |
| `Global`     | A name nothing in scope binds, such as `t_rt`                                                                      |
| `Unresolved` | Anything else                                                                                                      |

A parameter or local hides a top-level name of the same spelling, as in Lua.

## Shader sources and includes

`XRayShader::load` reads a source and every source it includes, recursively, for one `ShaderRenderer`.

Storage is the caller's. An `XRayShaderSourceLoader` returns `Ok(None)` for an absent path and `Err` for one that
exists but cannot be read, so a storage failure is never mistaken for an include fallback. Loose and archived bytes are
served alike.

Include paths resolve below the shaders root, never relative to the including file. The renderer decides the
candidates, tried in order:

| Renderer    | Include syntax                                          | Search order below the shaders root |
| ----------- | ------------------------------------------------------- | ----------------------------------- |
| `DirectX11` | A line starting `#include "path"` or `#include <path>`  | `r3/`, then the shaders root        |
| `OpenGl`    | Every `#include "path"` in a line, even one behind `//` | `gl/`                               |

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_shaders::{ShaderRenderer, XRayShader, XRayShaderSourceLoader};

struct MemoryLoader(HashMap<PathBuf, Vec<u8>>);

impl XRayShaderSourceLoader for MemoryLoader {
  fn load_source(&self, path: &Path) -> XrfResult<Option<Vec<u8>>> {
    Ok(self.0.get(path).cloned())
  }
}

fn main() -> XrfResult {
  let root: &Path = Path::new("shaders");
  let loader = MemoryLoader(HashMap::from([
    (root.join("r3/main.ps"), b"#include \"shared/common.h\"\n".to_vec()),
    (root.join("shared/common.h"), b"float value;\n".to_vec()),
  ]));
  let shader: XRayShader = XRayShader::load(root.join("r3/main.ps"), ShaderRenderer::DirectX11, root, &loader)?;

  // `r3/shared/common.h` is absent, so DirectX 11 falls back to the shaders root.
  assert_eq!(shader.imports()[0].path(), "shared/common.h");
  assert_eq!(shader.imports()[0].shader().path(), root.join("shared/common.h"));

  Ok(())
}
```

Errors:

- A missing source, or an include no candidate holds, is `XrfError::NotFound`, naming the including file and line.
- A malformed include (no quote or bracket, no closing delimiter, an empty path) is `XrfError::Invalid`.
- An include cycle is `XrfError::Verify`.
- A loader's own error passes through unchanged.

`XRayShader::is_source_path` recognizes a compiled source in a VFS listing by its extension, ignoring ASCII case, from
`XRayShader::SOURCE_FILE_EXTENSIONS` (`h`, `vs`, `ps`, `cs`, `hs`, `ds`, `gs`). `.s` scripts are not sources.

## Limitations

- Nothing compiles a shader here. `XRayShaderCompiler` is the interface for a caller-provided backend; the supplied
  `XRayShaderPlaceholderCompiler` always fails with `XrfError::NotImplemented`. Resolving includes does not prove that
  HLSL or GLSL compiles.
- `ShaderRenderer` covers DirectX 11 (`r3`) and OpenGL (`gl`) only, not the legacy `r1` and `r2` paths.

## Fixtures

The `fixtures` feature exports `xrf_shaders::fixtures` for dependants' tests:

- `ShaderBlenderFixture` builds blenders shaped like the shipped classes (`model`, `tree`, `detail`, `of(class, name)`,
  …);
- `ShaderLibraryFixture::library` writes a `shaders.xr` holding exactly those blenders.

## Checks

Run `cargo test --locked -p xrf-shaders`.

The crate is grouped by form: `src/library/` (the blender library, its properties under `property/`),
`src/compiler_table/`, `src/script/` and `src/source/`. Each re-exports its types, and the crate root re-exports them
flat.

Sources: the [library](src/library/shader_library_file.rs) and [blender](src/library/shader_blender.rs), the
[compiler table](src/compiler_table/shader_compiler_file.rs), the [script parser](src/script/xray_shader_script.rs), the
[source loader](src/source/xray_shader.rs), and the [compiler interface](src/source/xray_shader_compiler.rs).
