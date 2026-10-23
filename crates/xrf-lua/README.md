# xrf-lua

Checks LuaJIT syntax and extracts Lua method calls used by X-Ray tooling.

X-Ray runs gameplay `.script` files and renderer shader scripts (`.s`) through LuaJIT. `xrf-gamedata`'s `scripts`
check validates syntax with this crate, and `xrf-shaders` reads a shader script's passes from its method calls, such
as `shader:begin("vertex", "pixel"):blend(true, blend.srcalpha, blend.one)`. Parsing is static: scripts are never
executed.

## Read method calls

```rust
use xrf_lua::{XRayLuaBinding, XRayLuaScript, XRayLuaValue};

fn main() -> xrf_error::XrfResult {
  let script = XRayLuaScript::parse(
    "shaders/r3/example.s",
    r#"
local tex_base = "water\\water_water"

function normal(shader)
  shader:begin("vertex", "pixel")
    : blend (true, blend.srcalpha, blend.one)
  shader:sampler("s_base"):texture(tex_base)
end
"#,
  )?;

  let begin = script.method_calls("shader", "begin")[0];

  assert_eq!(begin.literal_string_arguments(), Some(vec![String::from("vertex"), String::from("pixel")]));
  assert_eq!(begin.function(), Some("normal"));

  let blend = begin.chained_call("blend").expect("a blend in the chain");

  assert!(blend.argument(0).is_some_and(XRayLuaValue::is_true));
  assert_eq!(blend.argument(1).and_then(XRayLuaValue::as_name), Some("blend.srcalpha"));

  let sampler = script.method_calls("shader", "sampler")[0];
  let texture = sampler.chained_call("texture").and_then(|call| call.argument(0));

  assert_eq!(
    texture,
    Some(&XRayLuaValue::Local {
      name: String::from("tex_base"),
      binding: XRayLuaBinding::String(String::from(r"water\water_water")),
    })
  );

  Ok(())
}
```

`XRayLuaScript::parse(path, source)` parses the supplied source string; the path only names the script in diagnostics
and is never opened. `method_calls(receiver, method)` answers every call of that method on a receiver named directly,
in source order.

## Call records

Each `XRayLuaMethodCall` exposes its receiver, method, 1-based `line_number`, arguments, the calls chained onto its
result (`chained`, `chained_call`), and the named `function` it is written in, `None` at the top level.
`literal_string_arguments` answers the arguments only when every one is a literal string.

Arguments are `XRayLuaValue`s, read as far as a reader that never runs the script can read them:

- literal strings (quoted escapes decoded as LuaJIT's lexer does, long brackets as written), numbers, and booleans;
- `Name`: a global or a dotted name such as `blend.srcalpha`, kept verbatim;
- `Local`: a bare name a local or parameter binds where it is written, with its `XRayLuaBinding`: a literal string, the
  argument at a parameter index (a method's implicit `self` is index 0), or anything else;
- `Other`: tables, operations, calls, `nil`, varargs, and strings with an escape LuaJIT would refuse.

Names resolve by Lua's scoping: the innermost local or parameter, a local only after the statement declaring it, a
`repeat` body's locals visible to its `until`, and a loop's variables invisible to its bounds. A local that any
assignment gives another value reads as `Other` everywhere, because the functions a script declares run after the
whole chunk and could see either value.

A syntactically valid script may still depend on missing globals or fail at runtime. Nothing here evaluates
expressions or follows control flow to decide which calls execute.

## Syntax-only checks

Call `verify_luajit_script(code, path)` when only validity matters; it parses the same way and discards the calls.

```rust
use std::path::Path;
use xrf_lua::verify_luajit_script;

fn main() -> xrf_error::XrfResult {
  verify_luajit_script("local enabled = true", Path::new("scripts/example.script"))?;
  assert!(verify_luajit_script("function broken(", Path::new("broken.script")).is_err());

  Ok(())
}
```

## Errors and checks

Syntax failures return `XrfError::Verify`, naming the path and listing the parser's diagnostics. Source decoding and
file reads belong to the caller. Run `cargo test --locked -p xrf-lua` from the repository root.

See the [script API](src/xray_lua_script.rs), [call record](src/xray_lua_method_call.rs), and
[argument values](src/xray_lua_value.rs).
