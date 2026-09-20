use std::path::{Path, PathBuf};

use full_moon::ast::Ast;
use full_moon::{LuaVersion, parse_fallible};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::lua_method_call_collector::LuaMethodCallCollector;
use crate::xray_lua_method_call::XRayLuaMethodCall;

/// A parsed LuaJIT script with normalized method calls.
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

  // A call outside every function belongs to none, which is what tells a script's own passes from its examples.
  #[test]
  fn leaves_a_call_at_the_top_level_without_a_function() -> XrfResult {
    let script: XRayLuaScript = XRayLuaScript::parse(Path::new("script.s"), "shader:begin(\"v\", \"p\")")?;

    assert_eq!(script.method_calls("shader", "begin")[0].function(), None);

    Ok(())
  }
}
