use std::path::{Path, PathBuf};

use full_moon::ast::Ast;
use full_moon::{LuaVersion, parse_fallible};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::lua_method_call_collector::LuaMethodCallCollector;
use crate::xray_lua_method_call::XRayLuaMethodCall;

/// A parsed LuaJIT script with normalized method calls, each argument read in the scope it is written in.
#[derive(Clone, Debug, PartialEq)]
pub struct XRayLuaScript {
  method_calls: Vec<XRayLuaMethodCall>,
  path: PathBuf,
}

impl XRayLuaScript {
  /// Parse LuaJIT source and retain the source path in any diagnostics.
  pub fn parse<P>(path: P, source: &str) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    let path: &Path = path.as_ref();
    let ast: Ast = Self::parse_ast(path, source)?;

    Ok(Self {
      method_calls: LuaMethodCallCollector::collect(&ast),
      path: path.to_path_buf(),
    })
  }

  pub fn method_calls(&self, receiver: &str, method: &str) -> Vec<&XRayLuaMethodCall> {
    self
      .method_calls
      .iter()
      .filter(|call| call.receiver() == receiver && call.method() == method)
      .collect()
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  fn parse_ast(path: &Path, source: &str) -> XrfResult<Ast> {
    parse_fallible(source, LuaVersion::luajit())
      .into_result()
      .map_err(|errors| {
        XrfError::new_verify_error(format!(
          "Failed to check LuaJIT script file: {}, errors: {}",
          format_path(path),
          errors.iter().map(|it| it.to_string()).collect::<Vec<_>>().join(", ")
        ))
      })
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use xrf_error::XrfResult;

  use super::XRayLuaScript;
  use crate::xray_lua_binding::XRayLuaBinding;
  use crate::xray_lua_chained_call::XRayLuaChainedCall;
  use crate::xray_lua_method_call::XRayLuaMethodCall;
  use crate::xray_lua_value::XRayLuaValue;

  #[test]
  fn collects_literal_and_dynamic_method_calls() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
shader:begin("vertex", "pixel")
shader:begin(dynamic_vertex, "dynamic_pixel")
other:begin("ignored_vertex", "ignored_pixel")
"#,
    )?;
    let shader_begins: Vec<&_> = script.method_calls("shader", "begin");

    assert_eq!(script.path(), Path::new("script.s"));
    assert_eq!(shader_begins.len(), 2);
    assert_eq!(
      shader_begins[0].literal_string_arguments(),
      Some(vec![String::from("vertex"), String::from("pixel")])
    );
    assert_eq!(shader_begins[1].literal_string_arguments(), None);

    Ok(())
  }

  #[test]
  fn collects_the_calls_chained_onto_one() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
function normal(shader)
  shader:begin("vertex", "pixel")
    : blend (true, blend.srcalpha, blend.one)
    : zb    (true, false)
    : aref  (true, 32)
end
"#,
    )?;
    let begin: &XRayLuaMethodCall = script.method_calls("shader", "begin")[0];

    assert_eq!(begin.function(), Some("normal"));
    assert_eq!(begin.chained().len(), 3);

    let blend: &XRayLuaChainedCall = begin.chained_call("blend").expect("a blend in the chain");

    assert!(blend.argument(0).expect("the switch").is_true());
    assert_eq!(
      blend.argument(1).and_then(XRayLuaValue::as_name),
      Some("blend.srcalpha")
    );
    assert_eq!(blend.argument(2).and_then(XRayLuaValue::as_name), Some("blend.one"));

    let zb: &XRayLuaChainedCall = begin.chained_call("zb").expect("a zb in the chain");

    assert!(zb.argument(0).expect("the test").is_true());
    assert!(!zb.argument(1).expect("the write").is_true());

    assert_eq!(
      begin
        .chained_call("aref")
        .and_then(|aref| aref.argument(1))
        .and_then(XRayLuaValue::as_number),
      Some(32.0)
    );

    Ok(())
  }

  /// What the first argument of each `:texture` chained onto a `shader:sampler` reads as, in source order.
  fn textures(script: &XRayLuaScript) -> Vec<XRayLuaValue> {
    script
      .method_calls("shader", "sampler")
      .iter()
      .filter_map(|call| {
        call
          .chained_call("texture")
          .and_then(|texture| texture.argument(0))
          .cloned()
      })
      .collect()
  }

  // `effects_water.s` names its textures once at the top and binds them in two functions.
  #[test]
  fn reads_a_name_bound_to_a_top_level_string_as_that_string() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("effects_water.s"),
      r#"
local tex_base = "water\\water_water"
local count = 3

function normal(shader)
  shader:sampler("s_base"):texture(tex_base)
  shader:sampler("s_count"):texture(count)
  shader:sampler("s_rt"):texture(t_rt)
end
"#,
    )?;

    assert_eq!(
      textures(&script),
      vec![
        XRayLuaValue::Local {
          binding: XRayLuaBinding::String(r"water\water_water".to_owned()),
          name: "tex_base".to_owned(),
        },
        XRayLuaValue::Local {
          binding: XRayLuaBinding::Other,
          name: "count".to_owned(),
        },
        XRayLuaValue::Name("t_rt".to_owned()),
      ]
    );

    Ok(())
  }

  // Lua resolves a name to its innermost declaration: a parameter or a local of the function hides the top-level one.
  #[test]
  fn reads_a_name_through_the_innermost_local_or_parameter_binding_it() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("effects_water.s"),
      r#"
local tex_base = "outer"

function normal(shader, tex_base)
  shader:sampler("s_parameter"):texture(tex_base)
end

local function helper(shader)
  local tex_base = "inner"
  shader:sampler("s_local"):texture(tex_base)

  if shader then
    local tex_base = "block"
  end

  shader:sampler("s_after_block"):texture(tex_base)
end

callback = function(shader)
  for _, tex_base in ipairs({}) do
    shader:sampler("s_loop"):texture(tex_base)
  end

  shader:sampler("s_top"):texture(tex_base)
end
"#,
    )?;
    let bindings: Vec<Option<XRayLuaBinding>> = textures(&script)
      .into_iter()
      .map(|value| match value {
        XRayLuaValue::Local { binding, .. } => Some(binding),
        _ => None,
      })
      .collect();

    assert_eq!(
      bindings,
      vec![
        Some(XRayLuaBinding::Parameter { index: 1 }),
        Some(XRayLuaBinding::String("inner".to_owned())),
        Some(XRayLuaBinding::String("inner".to_owned())),
        Some(XRayLuaBinding::Other),
        Some(XRayLuaBinding::String("outer".to_owned())),
      ]
    );

    Ok(())
  }

  // A local is in scope only after the statement declaring it, and a function declared before it reads the global.
  #[test]
  fn reads_a_name_declared_after_its_use_as_the_global() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
function normal(shader)
  shader:sampler("s_base"):texture(tex_base)
end

local tex_base = "late"
local tex_self = tex_self
shader:sampler("s_self"):texture(tex_self)
"#,
    )?;

    assert_eq!(
      textures(&script),
      vec![
        XRayLuaValue::Name("tex_base".to_owned()),
        XRayLuaValue::Local {
          binding: XRayLuaBinding::Other,
          name: "tex_self".to_owned(),
        },
      ]
    );

    Ok(())
  }

  /// What each `:texture` argument's binding is, `None` for one no local or parameter binds.
  fn bindings(script: &XRayLuaScript) -> Vec<Option<XRayLuaBinding>> {
    textures(script)
      .into_iter()
      .map(|value| match value {
        XRayLuaValue::Local { binding, .. } => Some(binding),
        _ => None,
      })
      .collect()
  }

  // Lua's `until` is inside the body's scope, and a loop's bounds are outside its variables.
  #[test]
  fn reads_a_repeat_condition_inside_its_body_and_a_loops_bounds_outside_it() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
local tex = "outer"

repeat
  local tex = "body"
until shader:sampler("s_until"):texture(tex)

shader:sampler("s_after_repeat"):texture(tex)

for tex = 1, shader:sampler("s_bound"):texture(tex) do
  shader:sampler("s_numeric"):texture(tex)
end

for _, tex in shader:sampler("s_iterated"):texture(tex) do
  shader:sampler("s_generic"):texture(tex)
end
"#,
    )?;

    assert_eq!(
      bindings(&script),
      vec![
        Some(XRayLuaBinding::String("body".to_owned())),
        Some(XRayLuaBinding::String("outer".to_owned())),
        Some(XRayLuaBinding::String("outer".to_owned())),
        Some(XRayLuaBinding::Other),
        Some(XRayLuaBinding::String("outer".to_owned())),
        Some(XRayLuaBinding::Other),
      ]
    );

    Ok(())
  }

  // `function object:method()` declares `self`; a label or a `goto` declares nothing.
  #[test]
  fn binds_self_in_a_method_and_nothing_for_a_label() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
local continue = "outer"

function blender:normal(shader)
  shader:sampler("s_self"):texture(self)
end

for index = 1, 2 do
  goto continue
  ::continue::
end

shader:sampler("s_after_label"):texture(continue)
"#,
    )?;

    assert_eq!(
      bindings(&script),
      vec![
        Some(XRayLuaBinding::Parameter { index: 0 }),
        Some(XRayLuaBinding::String("outer".to_owned())),
      ]
    );

    Ok(())
  }

  // A parameter is the argument at its position; a method's declared parameters come after its implicit `self`.
  #[test]
  fn binds_each_parameter_to_its_argument_position() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
function normal(shader, t_base, t_second)
  shader:sampler("s_second"):texture(t_second)
end

function blender:special(shader, t_base)
  shader:sampler("s_base"):texture(t_base)
end
"#,
    )?;

    assert_eq!(
      bindings(&script),
      vec![
        Some(XRayLuaBinding::Parameter { index: 2 }),
        Some(XRayLuaBinding::Parameter { index: 2 }),
      ]
    );

    Ok(())
  }

  // A reader walking in source order cannot tell which value an assigned local holds where it is read: the functions
  // a script declares run after the whole chunk, so a later assignment at the top reaches them too.
  #[test]
  fn reads_a_local_some_assignment_changes_as_nothing_it_knows() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
local tex_base = "first"
local tex_kept = "kept"
local helper

function normal(shader)
  shader:sampler("s_base"):texture(tex_base)
  shader:sampler("s_kept"):texture(tex_kept)
  shader:sampler("s_helper"):texture(helper)
end

tex_base = "second"

function helper() end
"#,
    )?;

    assert_eq!(
      bindings(&script),
      vec![
        Some(XRayLuaBinding::Other),
        Some(XRayLuaBinding::String("kept".to_owned())),
        Some(XRayLuaBinding::Other),
      ]
    );

    Ok(())
  }

  #[test]
  fn reads_nil_and_varargs_as_nothing_it_knows() -> XrfResult {
    let script: XRayLuaScript =
      XRayLuaScript::parse(Path::new("script.s"), "function f(...) shader:begin(nil, ...) end")?;

    assert_eq!(
      script.method_calls("shader", "begin")[0].arguments(),
      &[XRayLuaValue::Other, XRayLuaValue::Other]
    );

    Ok(())
  }

  #[test]
  fn reads_a_long_bracket_string_as_written() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(Path::new("script.s"), "shader:begin([[a\\b]], [==[\nfirst]==])")?;

    assert_eq!(
      script.method_calls("shader", "begin")[0].literal_string_arguments(),
      Some(vec![r"a\b".to_owned(), "first".to_owned()])
    );

    Ok(())
  }

  #[test]
  fn decodes_every_escape_a_quoted_string_may_carry() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"shader:begin("\a\b\f\n\r\t\v\\\"\'", "\65\066\0671", "\x41\u{42}\u{44f}", "a\z
          b", "tail\
next")"#,
    )?;

    assert_eq!(
      script.method_calls("shader", "begin")[0].literal_string_arguments(),
      Some(vec![
        "\u{7}\u{8}\u{c}\n\r\t\u{b}\\\"'".to_owned(),
        "ABC1".to_owned(),
        "AB\u{44f}".to_owned(),
        "ab".to_owned(),
        "tail\nnext".to_owned(),
      ])
    );

    Ok(())
  }

  #[test]
  fn reads_a_string_whose_escape_it_cannot_decode_as_nothing_it_knows() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(Path::new("script.s"), r#"shader:begin("\256", "\xZZ", "\q")"#)?;

    assert_eq!(
      script.method_calls("shader", "begin")[0].arguments(),
      &[XRayLuaValue::Other, XRayLuaValue::Other, XRayLuaValue::Other]
    );

    Ok(())
  }

  // A call outside every function belongs to none, which is what tells a script's own passes from its examples.
  #[test]
  fn leaves_a_call_at_the_top_level_without_a_function() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(Path::new("script.s"), "shader:begin(\"v\", \"p\")")?;

    assert_eq!(script.method_calls("shader", "begin")[0].function(), None);

    Ok(())
  }
}
