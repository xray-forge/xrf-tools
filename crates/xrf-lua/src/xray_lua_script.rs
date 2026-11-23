use std::path::{Path, PathBuf};

use full_moon::ast::Ast;
use full_moon::{LuaVersion, parse_fallible};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::lua_definition_collector::LuaDefinitionCollector;
use crate::lua_method_call_collector::LuaMethodCallCollector;
use crate::lua_string_use_collector::LuaStringUseCollector;
use crate::xray_lua_method_call::XRayLuaMethodCall;
use crate::xray_lua_string_call::XRayLuaStringCall;

/// A parsed LuaJIT script with normalized method calls, each argument read in the scope it is written in, and the
/// functions it defines for others to call by name.
#[derive(Clone, Debug, PartialEq)]
pub struct XRayLuaScript {
  method_calls: Vec<XRayLuaMethodCall>,
  global_functions: Vec<String>,
  registered_names: Vec<String>,
  string_calls: Vec<XRayLuaStringCall>,
  table_strings: Vec<String>,
  concatenation_prefixes: Vec<String>,
  path: PathBuf,
}

impl XRayLuaScript {
  /// Parse LuaJIT source and read its method calls; `path` only names the script in diagnostics.
  ///
  /// # Errors
  ///
  /// Returns a verify error naming `path` and listing the parser's diagnostics.
  pub fn parse<P>(path: P, source: &str) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    let path: &Path = path.as_ref();
    let ast: Ast = Self::parse_ast(path, source)?;
    let definitions: LuaDefinitionCollector = LuaDefinitionCollector::collect(&ast);
    let strings: LuaStringUseCollector = LuaStringUseCollector::collect(&ast);

    Ok(Self {
      method_calls: LuaMethodCallCollector::collect(&ast),
      global_functions: definitions.global_functions,
      registered_names: definitions.registered_names,
      string_calls: strings.calls,
      table_strings: strings.table_strings,
      concatenation_prefixes: strings.concatenation_prefixes,
      path: path.to_path_buf(),
    })
  }

  /// The non-local functions the script declares or assigns to a plain name, at any depth, which X-Ray reaches as
  /// `<file>.<function>` once it loads the script into the namespace its file names. A name an earlier `local`
  /// declares is listed too, so the list may hold more than the namespace does, never less.
  pub fn get_global_functions(&self) -> &[String] {
    &self.global_functions
  }

  /// The literal names the script registers through xrf-engine's `extern(name, value)`, each a dotted path under the
  /// global table such as `dialogs.break_dialog`.
  pub fn get_registered_names(&self) -> &[String] {
    &self.registered_names
  }

  /// The calls the script passes literal strings to, in source order, each with the function it names: the names it
  /// hands to functions that act on them, such as the info portions it gives and checks.
  pub fn get_string_calls(&self) -> &[XRayLuaStringCall] {
    &self.string_calls
  }

  /// The literal strings the script keeps in tables as data, keys and values, in source order, repeats kept:
  /// `{ ordered = "name" }` keeps a name for a later call, and `{ name = {...} }` keys data by one. A field whose key
  /// spells its value, `name = "name"`, only mirrors a constant and is left out.
  pub fn get_table_strings(&self) -> &[String] {
    &self.table_strings
  }

  /// The literal strings the script builds names from, opening a chain of `..`, in source order, repeats kept:
  /// `"zat_b51_ordered_item_" .. index` builds every name the prefix opens, while the literal of `a .. "_b_" .. c`
  /// starts in the middle of one and is left out.
  pub fn get_concatenation_prefixes(&self) -> &[String] {
    &self.concatenation_prefixes
  }

  /// Whether LuaJIT source parses, for a caller that only needs to know the script is valid.
  ///
  /// # Errors
  ///
  /// Returns a verify error naming `path` and listing the parser's diagnostics.
  pub fn verify<P>(path: P, source: &str) -> XrfResult
  where
    P: AsRef<Path>,
  {
    Self::parse_ast(path.as_ref(), source).map(drop)
  }

  /// Every call of `method` on a receiver named `receiver` directly, in source order.
  pub fn method_calls(&self, receiver: &str, method: &str) -> Vec<&XRayLuaMethodCall> {
    self
      .method_calls
      .iter()
      .filter(|call| call.receiver() == receiver && call.method() == method)
      .collect()
  }

  /// The path the script was parsed as, which names it in diagnostics.
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
  use crate::xray_lua_string_call::XRayLuaStringCall;
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

  // A local function is as much a named function as a global one, and the call after it is back at the top.
  #[test]
  fn names_the_local_function_a_call_is_written_in() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("script.s"),
      r#"
local function helper(shader)
  shader:begin("v", "p")
end

shader:begin("top", "level")
"#,
    )?;
    let begins: Vec<&XRayLuaMethodCall> = script.method_calls("shader", "begin");

    assert_eq!(begins[0].function(), Some("helper"));
    assert_eq!(begins[1].function(), None);

    Ok(())
  }

  #[test]
  fn reads_decimal_hexadecimal_and_exponent_numbers() -> XrfResult {
    let script: XRayLuaScript =
      XRayLuaScript::parse(Path::new("script.s"), "shader:begin(32, 0x80, 0XfF, 1.5e2, .5, 10LL)")?;
    let numbers: Vec<Option<f64>> = script.method_calls("shader", "begin")[0]
      .arguments()
      .iter()
      .map(XRayLuaValue::as_number)
      .collect();

    assert_eq!(
      numbers,
      vec![Some(32.0), Some(128.0), Some(255.0), Some(150.0), Some(0.5), None]
    );

    Ok(())
  }

  // A script's top level is what X-Ray runs when it loads the script into its namespace.
  #[test]
  fn lists_the_functions_the_top_level_defines_and_the_names_it_registers() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(
      Path::new("dialogs.script"),
      r#"
function break_dialog(first_speaker, second_speaker) end
is_wounded = function(actor, npc) end
local function helper() end
local cached = function() end
function object:method() end
function module.member() end
count = 3

function register()
  late = function() end
  extern("dialogs_zaton.zat_b30_owl_has_stash", function() end)
  extern(name_of_something, function() end)
end

extern("travel_callbacks", { can_travel = function() end, ["get_cost"] = function() end, 3 })
"#,
    )?;

    // A function a body defines lands in the namespace too, once the body runs.
    assert_eq!(
      script.get_global_functions(),
      ["break_dialog", "is_wounded", "register", "late"]
    );
    assert_eq!(
      script.get_registered_names(),
      [
        "dialogs_zaton.zat_b30_owl_has_stash",
        "travel_callbacks.can_travel",
        "travel_callbacks.get_cost",
        "travel_callbacks"
      ]
    );

    Ok(())
  }

  /// Names reach the functions acting on them as call arguments, each call naming its method, its dotted field or its
  /// bare name, as values a table keeps for later, or built onto a literal prefix, while a field mirroring its own key
  /// is only a constant.
  #[test]
  fn collects_the_literal_strings_used_as_names() -> XrfResult {
    let code: &str = r#"
local names = { known = "known_table_value", mirrored = "mirrored", [1] = "indexed_value", "listed_value" }

db.actor:give_info_portion("given_by_method")
if has_alife_info("read_by_call") and hasInfoPortion(names.known) then
  print "bare_string_call"
end
xr_effects.give_info(actor, nil, "dotted_call")
handlers[key]("unnamed_call")
local built = "built_prefix_" .. index .. "_suffix"
local inner = name .. "middle_part_" .. index
"#;
    let script: XRayLuaScript = XRayLuaScript::parse("info.script", code)?;
    let call = |function: Option<&str>, argument: &str| -> XRayLuaStringCall {
      XRayLuaStringCall::from_parts(function.map(str::to_owned), vec![String::from(argument)])
    };

    assert_eq!(
      script.get_string_calls(),
      [
        call(Some("give_info_portion"), "given_by_method"),
        call(Some("has_alife_info"), "read_by_call"),
        call(Some("print"), "bare_string_call"),
        call(Some("give_info"), "dotted_call"),
        call(None, "unnamed_call"),
      ]
    );
    assert_eq!(
      script.get_table_strings(),
      ["known", "known_table_value", "indexed_value", "listed_value"]
    );
    assert_eq!(script.get_concatenation_prefixes(), ["built_prefix_"]);

    Ok(())
  }

  #[test]
  fn accepts_luajit_goto_and_label() {
    let code: &str = r#"
local ____exports = {}

for index = 1, 3 do
  if index == 2 then
    goto __continue1
  end

  ::__continue1::
end

return ____exports
"#;

    assert!(XRayLuaScript::verify(Path::new("generated.script"), code).is_ok());
  }

  #[test]
  fn reports_invalid_luajit_syntax_with_source_path() {
    let error: String = XRayLuaScript::verify(Path::new("invalid.script"), "local value =")
      .expect_err("Expected malformed LuaJIT script to fail")
      .to_string();

    assert!(error.contains("invalid.script"));
    assert!(error.contains("errors:"));
  }
}
