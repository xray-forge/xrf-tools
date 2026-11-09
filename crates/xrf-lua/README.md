# xrf-lua

Checks LuaJIT syntax and extracts Lua method calls used by X-Ray tooling.

X-Ray runs gameplay `.script` files and renderer shader scripts (`.s`) through LuaJIT.

- `xrf-gamedata`'s `scripts` check validates syntax with this crate.
- `xrf-shaders` reads a shader script's passes from its method calls, such as
  `shader:begin("vertex", "pixel"):blend(true, blend.srcalpha, blend.one)`.

Parsing is static: scripts are never run.

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

Each `XRayLuaMethodCall` exposes:

- its receiver, method and 1-based `line_number`;
- its arguments; `literal_string_arguments` answers them only when every one is a literal string;
- the calls chained onto its result (`chained`, `chained_call`);
- the named `function` it is written in, `local function` included; `None` at the top level or in an anonymous
  function.

Arguments are `XRayLuaValue`s, read as far as possible without running the script:

- literal strings (quoted escapes decoded as LuaJIT's lexer does, long brackets as written), numbers (decimal, with a
  fraction or exponent, or a hexadecimal integer), and booleans;
- `Name`: a global or a dotted name such as `blend.srcalpha`, kept verbatim;
- `Local`: a bare name a local or parameter binds where it is written, with its `XRayLuaBinding`: a literal string, the
  argument at a parameter index (a method's implicit `self` is index 0), or anything else;
- `Other`: tables, operations (a negative number among them), calls, `nil`, varargs, numbers only LuaJIT's FFI reads
  (`10LL`, `1i`), and strings with an escape LuaJIT would refuse.

Names resolve by Lua's scoping rules:

- the innermost local or parameter wins;
- a local is visible only after the statement declaring it;
- a `repeat` body's locals are visible to its `until`;
- a loop's variables are not visible to its bounds.

A local that any assignment changes reads as `Other` everywhere, because the script's functions run after the whole
chunk and could see either value.

Nothing evaluates expressions or follows control flow to decide which calls run. A syntactically valid script may still
depend on missing globals or fail at runtime.

## Syntax-only checks

Call `XRayLuaScript::verify(path, source)` when only validity matters; it parses the same way and reads no calls.

```rust
use xrf_lua::XRayLuaScript;

fn main() -> xrf_error::XrfResult {
  XRayLuaScript::verify("scripts/example.script", "local enabled = true")?;
  assert!(XRayLuaScript::verify("broken.script", "function broken(").is_err());

  Ok(())
}
```

## Errors

Syntax failures return `XrfError::Verify`, naming the path and listing the parser's diagnostics. Reading and decoding
the source is the caller's job.

## Checks

Run `cargo test --locked -p xrf-lua` from the repository root.

Sources: the [script API](src/xray_lua_script.rs), [call record](src/xray_lua_method_call.rs), and
[argument values](src/xray_lua_value.rs).
