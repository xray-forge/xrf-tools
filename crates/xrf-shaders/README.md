# xrf-shaders

Parses X-Ray Lua shader scripts and resolves shader source imports for a selected renderer.

## Read a shader script

```rust
use xrf_shaders::XRayShaderScript;

fn main() -> xrf_error::XrfResult {
  let script = XRayShaderScript::parse(
    "shaders/r3/example.s",
    "function normal(shader) shader:begin(\"vertex\", \"pixel\") end",
  )?;

  assert_eq!(script.passes().len(), 1);
  assert_eq!(script.passes()[0].vertex_shader(), "vertex");
  assert_eq!(script.passes()[0].pixel_shader(), "pixel");

  Ok(())
}
```

Pass collection selects `shader:begin` calls with exactly two literal string arguments. Computed names are not evaluated,
and calls in comments do not produce passes.

## Load shader sources

Parse a `.s` script with `XRayShaderScript::parse` and inspect `passes`. Load a shader source tree with `XRayShader::load`,
passing a `ShaderRenderer`, shaders root, and an implementation of `XRayShaderSourceLoader`.

The loader returns `Ok(None)` for an absent source and `Err` for read failures. This lets the caller supply loose or
archived source bytes without putting storage policy in the shader parser.

`XRayShader::source` exposes the loaded source bytes and `imports` exposes resolved imports. Renderer selection controls
the include candidates:

| Renderer    | Search order below the shaders root |
| ----------- | ----------------------------------- |
| `DirectX11` | `r3/`, then the shaders root        |
| `OpenGl`    | `gl/`                               |

The loader receives candidate paths. It must distinguish an absent source from a source that exists but cannot be read;
otherwise a storage failure can be mistaken for an include fallback.

## Compilation and limitations

Shader compilation is not implemented by the supplied `XRayShaderPlaceholderCompiler`: it always returns
`XrfError::NotImplemented`. `XRayShaderCompiler` is the interface for a future or caller-provided backend.

The available renderer variants do not cover the legacy `r1` and `r2` paths. Parsing a Lua pass or resolving includes
does not prove that HLSL or GLSL compiles, and the Lua script is not executed.

## Errors and checks

Lua syntax failures propagate through the parser. Source-read failures propagate from the loader; invoking the supplied
placeholder compiler always fails. Run `cargo test --locked -p xrf-shaders` from the repository root.

See the [script parser](src/xray_shader_script.rs), [source API](src/xray_shader.rs), and
[compiler interface](src/xray_shader_compiler.rs). Binary `shaders.xr` libraries belong to [xrf-db](../xrf-db/README.md).
