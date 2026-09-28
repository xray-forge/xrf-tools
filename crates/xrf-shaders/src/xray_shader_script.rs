use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_lua::{XRayLuaMethodCall, XRayLuaScript};

use crate::{XRayShaderPass, XRayShaderSampler};

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
    let begins: Vec<&XRayLuaMethodCall> = lua_script.method_calls("shader", "begin");
    let samplers: Vec<&XRayLuaMethodCall> = lua_script.method_calls("shader", "sampler");
    let passes: Vec<XRayShaderPass> = begins
      .iter()
      .filter_map(|method_call| {
        let arguments: Vec<String> = method_call.literal_string_arguments()?;
        let [vertex_shader, pixel_shader] = arguments.as_slice() else {
          return None;
        };
        // A pass binds what its function binds after it and before the function begins another.
        let from: usize = method_call.line_number();
        let to: usize = begins
          .iter()
          .filter(|next| next.function() == method_call.function() && next.line_number() > from)
          .map(|next| next.line_number())
          .min()
          .unwrap_or(usize::MAX);
        let bound: Vec<XRayShaderSampler> = samplers
          .iter()
          .filter(|sampler| {
            sampler.function() == method_call.function() && sampler.line_number() >= from && sampler.line_number() < to
          })
          .filter_map(|sampler| XRayShaderSampler::of(sampler))
          .collect();

        Some(XRayShaderPass::of(method_call, vertex_shader.clone(), pixel_shader.clone()).with_samplers(bound))
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
  use crate::xray_shader_sampler_texture::XRayShaderSamplerTexture;

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

  // `shaders/r2/effects_water.s`: its textures are named once at the top, each pass binds its own.
  #[test]
  fn reads_the_samplers_each_pass_binds() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r2/effects_water.s"),
      r#"
local tex_nmap                = "water\\water_normal"
local tex_dist                = "water\\water_dudv"
local tex_env0                = "$user$sky0"

function normal                (shader, t_base, t_second, t_detail)
  shader:begin                ("water_soft","water_soft")
        : blend                (true,blend.srcalpha,blend.invsrcalpha)
  shader:sampler        ("s_base")       :texture  (t_base)
  shader:sampler        ("s_nmap")       :texture  (tex_nmap)
  shader:sampler        ("s_env0")       :texture  (tex_env0)   : clamp()
  shader:sampler        ("s_leaves")     :texture  ("water\\water_foam") : wrap()
end

function l_special        (shader, t_base, t_second, t_detail)
  shader:begin                ("waterd_soft","waterd_soft")
  shader:sampler        ("s_distort")    :texture  (tex_dist)
end
"#,
    )?;

    let normal: &XRayShaderPass = script.pass_of(XRayShaderPass::BASE_FUNCTION).expect("a base pass");
    let special: &XRayShaderPass = script.pass_of("l_special").expect("a distortion pass");
    let texture = |pass: &XRayShaderPass, name: &str| pass.sampler(name).map(|sampler| sampler.texture().clone());

    assert_eq!(normal.samplers().len(), 4);
    assert_eq!(
      texture(normal, "s_base"),
      Some(XRayShaderSamplerTexture::Parameter("t_base".to_owned()))
    );
    assert_eq!(
      texture(normal, "s_nmap"),
      Some(XRayShaderSamplerTexture::Named(r"water\water_normal".to_owned()))
    );
    assert_eq!(normal.sampler("s_env0").and_then(|it| it.texture().file()), None);
    assert_eq!(
      normal.sampler("s_leaves").and_then(|it| it.texture().file()),
      Some(r"water\water_foam")
    );
    assert_eq!(
      texture(special, "s_distort"),
      Some(XRayShaderSamplerTexture::Named(r"water\water_dudv".to_owned()))
    );
    assert!(special.sampler("s_nmap").is_none());

    Ok(())
  }

  // A parameter or a local of the pass's function hides a top-level name of the same spelling, as it does in Lua.
  #[test]
  fn reads_a_samplers_texture_through_the_binding_nearest_it() -> XrfResult {
    let script: XRayShaderScript = XRayShaderScript::parse(
      Path::new("shaders/r2/effects_shadowed.s"),
      r#"
local t_base   = "outer\\base"
local tex_nmap = "outer\\normal"

function normal(shader, t_base, t_second, t_detail)
  local tex_nmap = "inner\\normal"

  shader:begin("model_def_lq", "model_def_lq")
  shader:sampler("s_base"):texture(t_base)
  shader:sampler("s_nmap"):texture(tex_nmap)
  shader:sampler("s_image"):texture(t_rt)
end
"#,
    )?;

    let normal: &XRayShaderPass = script.pass_of(XRayShaderPass::BASE_FUNCTION).expect("a base pass");
    let texture = |name: &str| normal.sampler(name).map(|sampler| sampler.texture().clone());

    assert_eq!(
      texture("s_base"),
      Some(XRayShaderSamplerTexture::Parameter("t_base".to_owned()))
    );
    assert_eq!(
      texture("s_nmap"),
      Some(XRayShaderSamplerTexture::Named(r"inner\normal".to_owned()))
    );
    assert_eq!(
      texture("s_image"),
      Some(XRayShaderSamplerTexture::Global("t_rt".to_owned()))
    );

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
