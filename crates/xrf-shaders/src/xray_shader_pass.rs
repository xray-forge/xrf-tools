use xrf_lua::XRayLuaMethodCall;

use crate::xray_shader_pass_state::XRayShaderPassState;

/// A literal vertex and pixel shader pair selected by `shader:begin`, and the state chained onto it.
#[derive(Clone, Debug, PartialEq)]
pub struct XRayShaderPass {
  /// The function the pass was declared in, which is what decides when the engine compiles it.
  function: Option<String>,
  line_number: usize,
  pixel_shader: String,
  state: XRayShaderPassState,
  vertex_shader: String,
}

impl XRayShaderPass {
  /// The function whose pass the renderer compiles as a surface's base element (`_lua_Create`).
  pub const BASE_FUNCTION: &'static str = "normal";

  /// Reads one pass off a literal `shader:begin` and whatever is chained onto it.
  pub fn of(call: &XRayLuaMethodCall, vertex_shader: String, pixel_shader: String) -> Self {
    Self {
      function: call.function().map(str::to_owned),
      line_number: call.line_number(),
      pixel_shader,
      state: XRayShaderPassState::of(call),
      vertex_shader,
    }
  }

  pub fn function(&self) -> Option<&str> {
    self.function.as_deref()
  }

  pub fn line_number(&self) -> usize {
    self.line_number
  }

  pub fn pixel_shader(&self) -> &str {
    &self.pixel_shader
  }

  pub fn state(&self) -> &XRayShaderPassState {
    &self.state
  }

  pub fn vertex_shader(&self) -> &str {
    &self.vertex_shader
  }
}
