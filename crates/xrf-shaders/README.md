# xrf-shaders

Reads X-Ray shaders in the three forms the game ships them: the compiled blender library `shaders.xr`, the level
compiler's table `shaders_xrlc.xr`, and the authored sources under `shaders\` (`.s` Lua scripts and the HLSL or GLSL
sources they name, with their includes). [xrf-material](../xrf-material/README.md) resolves what a surface looks like
from the library and the scripts, `xrf-gamedata` verifies the library, the scripts and the sources, and the desktop app
describes both archived tables.

## The blender library

A shader name that a mesh, a level surface or a config declares, such as `models\model_aref`, resolves to one compiled
blender of `shaders.xr`. `ShaderLibraryFile` reads the library from a path, an open file, bytes already in hand (an
archived library has no file to open), or a chunk reader, always in X-Ray's byte order. It looks a blender up by its
exact name (`find_blender`, `contains_blender`); `blenders` iterates in no particular order.

Reading refuses a library without its blenders chunk (id 2), a name defined twice, a blender chunk not consumed whole,
and a property type the engine never writes (11 and above, `XrfError::NotImplemented`), since a guessed payload size
would misread every property after it.

A `ShaderBlender` holds the description `CBlender_DESC` lays out — its class, name, the computer and time the SDK saved
it at, and the class's format version — then its properties in the order the class wrote them:

- `class` is a `ShaderBlenderClass`, the engine's eight-character tag packed into a `u64`; constants name the known ones
  (`MODEL`, `DEFAULT_AREF`, `TREE`, …) and `tag()` prints one without its padding.
- A property is a name and a `ShaderBlenderPropertyValue` of one of eleven kinds. Markers head groups: `group(marker)`
  answers the properties up to the next marker, and `texture(marker, name)` one texture in it.
- `integer`, `boolean` and `token` read a knob by name, `None` when the class does not write it or it is of another
  kind.
- `base_texture(textures)` answers the texture the base slot comes to for a surface compiled against that texture list:
  a `$base<N>` spelling (`N` up to 7) indexes the list, `$null` or an empty name is no texture, and anything else names
  itself.

`ShaderBlender` and `ShaderBlenderProperty` implement `xrf_chunk::ChunkReadWrite` and write back byte for byte; the
library itself has no writer. Fixed-width text fields are Windows-1251 and null terminated, and a value too long for
its field is refused on write.

## The compiler table

`ShaderCompilerFile` reads `shaders_xrlc.xr`, the per-shader settings the level compiler bakes with: a plain array of
144-byte `ShaderCompilerShader` records (a 128-byte name, flags, vertex translucency, vertex ambient, and lightmap
density in texels per unit). Its readers take the byte order as a type parameter; callers pass `XRayByteOrder`. A size
that is not a whole number of records is refused, as the engine asserts.

The bytes the editor left in a name field after its terminator are kept (`name_trailing`), so `write` reproduces the
file byte for byte and refuses a name changed without them. `find_shader` matches a name ASCII case-insensitively, as
the compiler does, and `get_named_flags` names the set bits from `SHADER_COMPILER_FLAGS`. Both types serialize with
serde, in camelCase.

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

- A pass is a `shader:begin` call on the `shader` receiver with exactly two literal string arguments. Computed names
  are not evaluated, and calls in comments, including a commented-out `--[[ ]]` draft, produce no pass. A Lua syntax
  error is `XrfError::Verify`.
- The function a pass is declared in decides when the engine compiles it: `XRayShaderPass::BASE_FUNCTION` (`normal`)
  is a surface's base element, `SPECIAL_FUNCTION` (`l_special`) its distortion or emissive element, and
  `SHADOW_FUNCTION` (`l_point`) the deferred renderers' shadow element. `pass_of(function)` answers that function's
  first pass, `None` when the script does not declare it.
- `state()` is the render state chained onto the `begin`: `blend`, `aref`, `zb`, `wmark`, `distort` and `emissive`. A
  pass that chains nothing tests and writes depth and does not blend.
- `samplers()` are the `shader:sampler` calls after the `begin` and before the function's next one, in source order.
  Each texture is an `XRayShaderSamplerTexture`: `Named` (a literal or a local bound to one, which may be an engine
  target such as `$user$sky0`; `file()` answers only real files), `Parameter` (an argument the renderer passes, such as
  `t_base`), `Global` (a name nothing in scope binds, such as `t_rt`), or `Unresolved`. A parameter or local hides a
  top-level name of the same spelling, as in Lua.

## Shader sources and includes

`XRayShader::load` reads a source and every source it includes, recursively, for one `ShaderRenderer`. Storage is the
caller's: an `XRayShaderSourceLoader` answers `Ok(None)` for an absent path and `Err` for one that exists but cannot be
read, so a storage failure is never mistaken for an include fallback. The loader serves loose or archived bytes alike.

An include path resolves below the shaders root, never relative to the including file; the renderer decides the
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

A missing source is `XrfError::NotFound`, and so is an include no candidate holds, naming the including file and line.
A malformed include (no quote or bracket, no closing delimiter, an empty path) is `XrfError::Invalid`, and an include
cycle `XrfError::Verify`. A loader's own error propagates unchanged.

`is_shader_source_path` tells a VFS listing's name apart as a compiled source by its extension, ASCII
case-insensitively, against `SHADER_SOURCE_FILE_EXTENSIONS` (`h`, `vs`, `ps`, `cs`, `hs`, `ds`, `gs`); `.s` scripts are
not sources.

## Limitations

- Nothing compiles a shader here. `XRayShaderCompiler` is the interface for a caller-provided backend, and the supplied
  `XRayShaderPlaceholderCompiler` always fails with `XrfError::NotImplemented`. Resolving includes does not prove that
  HLSL or GLSL compiles.
- `ShaderRenderer` covers DirectX 11 (`r3`) and OpenGL (`gl`) only, not the legacy `r1` and `r2` paths.

## Fixtures

The `fixtures` feature exports `xrf_shaders::fixtures` for dependants' tests: `ShaderBlenderFixture` builds blenders of
the shapes the shipped classes write (`model`, `tree`, `detail`, `of(class, name)`, …), and
`ShaderLibraryFixture::library` writes a `shaders.xr` holding exactly those blenders.

## Checks

Run `cargo test --locked -p xrf-shaders`. Sources: the [library](src/shader_library_file.rs) and
[blender](src/shader_blender.rs), the [compiler table](src/shader_compiler_file.rs), the
[script parser](src/xray_shader_script.rs), the [source loader](src/xray_shader.rs), and the
[compiler interface](src/xray_shader_compiler.rs).
