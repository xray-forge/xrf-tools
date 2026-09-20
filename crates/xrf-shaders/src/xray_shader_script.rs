use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_lua::XRayLuaScript;

use crate::XRayShaderPass;

/// An X-Ray renderer shader script and the literal passes it declares.
#[derive(Clone, Debug, PartialEq)]
pub struct XRayShaderScript {
  passes: Vec<XRayShaderPass>,
  path: PathBuf,
}

impl XRayShaderScript {
  /// Parse an X-Ray shader script and collect literal `shader:begin` calls.
  pub fn parse<P>(path: P, source: &str) -> XrfResult<Self>
  where
    P: AsRef<Path>,
  {
    let path: &Path = path.as_ref();
    let lua_script: XRayLuaScript = XRayLuaScript::parse(path, source)?;
    let passes: Vec<XRayShaderPass> = lua_script
      .method_calls("shader", "begin")
      .into_iter()
      .filter_map(|method_call| {
        let arguments: Vec<String> = method_call.literal_string_arguments()?;
        let [vertex_shader, pixel_shader] = arguments.as_slice() else {
          return None;
        };

        Some(XRayShaderPass::of(
          method_call,
          vertex_shader.clone(),
          pixel_shader.clone(),
        ))
      })
      .collect();

    Ok(Self {
      passes,
      path: path.to_path_buf(),
    })
  }

  pub fn passes(&self) -> &[XRayShaderPass] {
    &self.passes
  }

  /// The first pass one of the script's functions declares, for a reader that wants the one the engine compiles.
  ///
  /// @param function - Name of the function, `normal` for the base element every surface is drawn by.
  pub fn pass_of(&self, function: &str) -> Option<&XRayShaderPass> {
    self.passes.iter().find(|pass| pass.function() == Some(function))
  }

  pub fn path(&self) -> &Path {
    &self.path
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use xrf_error::{XrfError, XrfResult};

  use super::XRayShaderScript;
  use crate::xray_shader_blend_factor::XRayShaderBlendFactor;
  use crate::xray_shader_pass::XRayShaderPass;
  use crate::xray_shader_pass_state::XRayShaderPassState;

  #[test]
  fn collects_literal_shader_passes() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r3/example.s"),
      r#"
function normal(shader)
  shader:begin("vertex", "pixel"):sorting(1, false)
  shader:begin(dynamic_vertex, "dynamic_pixel")
  other:begin("ignored_vertex", "ignored_pixel")
  -- shader:begin("commented_vertex", "commented_pixel")
end
"#,
    )?;

    assert_eq!(script.path(), Path::new("shaders/r3/example.s"));
    assert_eq!(script.passes().len(), 1);
    assert_eq!(script.passes()[0].vertex_shader(), "vertex");
    assert_eq!(script.passes()[0].pixel_shader(), "pixel");

    Ok(())
  }

  #[test]
  fn reports_luajit_syntax_errors() {
    let result = XRayShaderScript::parse(Path::new("invalid.s"), "function normal(");

    assert!(matches!(result, Err(XrfError::Verify { .. })));
  }
  // `shaders/r2/effects_lightplanes.s`, whose commented-out first draft is the trap: read it and the pass reads as
  // `zero, one`, which draws nothing at all.
  #[test]
  fn reads_the_state_of_the_pass_the_engine_compiles() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r2/effects_lightplanes.s"),
      r#"
--[[
function normal		(shader, t_base, t_second, t_detail)
	shader:begin	("dumb","dumb")
			: blend		(true,blend.zero,blend.one)
end
]]

function normal		(shader, t_base, t_second, t_detail)
	shader:begin	("base_lplanes","base_lplanes")
			: fog		(false)
			: zb 		(true,false)
			: blend		(true,blend.srcalpha,blend.one)
			: aref 		(true,0)
			: sorting	(2, false)
	shader:sampler	("s_base")      :texture	(t_base)
end
"#,
    )?;

    let pass: &XRayShaderPass = script.pass_of(XRayShaderPass::BASE_FUNCTION).expect("a base pass");
    let state: &XRayShaderPassState = pass.state();

    assert_eq!(pass.vertex_shader(), "base_lplanes");
    assert!(state.is_blended);
    assert_eq!(state.blend_source, Some(XRayShaderBlendFactor::SourceAlpha));
    assert_eq!(state.blend_destination, Some(XRayShaderBlendFactor::One));
    assert!(state.is_alpha_tested);
    assert_eq!(state.alpha_reference, Some(0));
    assert!(state.is_depth_tested);
    assert!(!state.is_depth_written);
    assert!(!state.is_wallmark);

    Ok(())
  }

  // `shaders/r2/effects_wallmarkblend.s`, which is how a mark laid on a wall is drawn.
  #[test]
  fn reads_a_wall_mark_as_one() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r2/effects_wallmarkblend.s"),
      r#"
function normal		(shader, t_base, t_second, t_detail)
	shader:begin	("wmark",	"simple")
			: blend		(true,blend.srcalpha,blend.invsrcalpha)
			: aref 		(true,0)
			: zb 		(true,false)
			: wmark		(true)
end
"#,
    )?;

    let state: &XRayShaderPassState = script
      .pass_of(XRayShaderPass::BASE_FUNCTION)
      .expect("a base pass")
      .state();

    assert_eq!(state.blend_source, Some(XRayShaderBlendFactor::SourceAlpha));
    assert_eq!(state.blend_destination, Some(XRayShaderBlendFactor::InverseSourceAlpha));
    assert!(state.is_wallmark);

    Ok(())
  }

  // A script whose only function is `l_special` declares no base element, so there is no pass to read for one.
  #[test]
  fn answers_nothing_for_a_function_the_script_does_not_declare() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r2/details_lod.s"),
      r#"
function l_special	(shader, t_base, t_second, t_detail)
	shader:begin	("lod","lod") : blend (false, blend.one, blend.zero) : zb (true, true)
end
"#,
    )?;

    assert!(script.pass_of(XRayShaderPass::BASE_FUNCTION).is_none());
    assert!(script.pass_of("l_special").is_some());

    Ok(())
  }

  // Nothing chained means the defaults the renderer starts a pass with: written, tested, not composited.
  #[test]
  fn reads_a_bare_pass_as_the_state_a_pass_starts_in() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r2/bare.s"),
      "function normal(shader) shader:begin(\"v\", \"p\") end",
    )?;

    let state: &XRayShaderPassState = script.pass_of("normal").expect("a base pass").state();

    assert!(!state.is_blended);
    assert!(state.is_depth_tested);
    assert!(state.is_depth_written);

    Ok(())
  }
}
