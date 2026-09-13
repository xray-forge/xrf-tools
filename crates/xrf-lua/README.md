# xrf-lua

Checks LuaJIT syntax and extracts Lua method calls used by X-Ray tooling.

## Read method calls

```rust
use xrf_lua::XRayLuaScript;

fn main() -> xrf_error::XrfResult {
  let script = XRayLuaScript::parse(
    "shaders/r3/example.s",
    "function normal(shader) shader:begin(\"vertex\", \"pixel\") end",
  )?;

  let calls = script.method_calls("shader", "begin");

  assert_eq!(calls.len(), 1);
  assert_eq!(calls[0].literal_string_arguments().unwrap(), &["vertex", "pixel"]);

  Ok(())
}
```

The path supplies diagnostic context; parsing uses the supplied source string and does not open that path.

## Syntax-only checks

Call `verify_luajit_script(code, path)` for syntax validation. Use `XRayLuaScript::parse(path, source)` when you also
need calls, then query `method_calls(receiver, method)`.

```rust
use std::path::Path;
use xrf_lua::verify_luajit_script;

fn main() -> xrf_error::XrfResult {
  verify_luajit_script("local enabled = true", Path::new("scripts/example.script"))?;
  assert!(verify_luajit_script("function broken(", Path::new("broken.script")).is_err());

  Ok(())
}
```

## Call records and limitations

Each `XRayLuaMethodCall` exposes its receiver, method, source line, and optional literal string arguments. Calls with
computed arguments cannot provide a literal argument list. This is static parsing; scripts are not executed.

Use `line_number` to place a diagnostic and `receiver` / `method` to identify the call. A syntactically valid script
may still depend on missing globals or fail at runtime. Literal collection does not evaluate expressions or follow
the runtime control flow to decide which calls execute.

## Errors and checks

Syntax validation failures return `XrfError::Verify` with parser diagnostics. Source decoding and file reads belong to
the caller. Run `cargo test --locked -p xrf-lua` from the repository root.

See the [script API](src/xray_lua_script.rs) and [call record](src/xray_lua_method_call.rs).
