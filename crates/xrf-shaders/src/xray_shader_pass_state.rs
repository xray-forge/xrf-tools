use xrf_lua::{XRayLuaChainedCall, XRayLuaMethodCall, XRayLuaValue};

use crate::xray_shader_blend_factor::XRayShaderBlendFactor;

/// The render state a shader script sets on one pass, in the calls chained onto its `shader:begin`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct XRayShaderPassState {
  /// Whether the pass is composited rather than written, `blend(true, ...)`.
  pub is_blended: bool,
  pub blend_source: Option<XRayShaderBlendFactor>,
  pub blend_destination: Option<XRayShaderBlendFactor>,
  /// Whether the pass discards texels below its reference, `aref(true, n)`.
  pub is_alpha_tested: bool,
  pub alpha_reference: Option<u8>,
  /// Whether the pass tests depth, the first argument of `zb`.
  pub is_depth_tested: bool,
  /// Whether the pass writes depth, the second argument of `zb`.
  pub is_depth_written: bool,
  /// Whether the pass is a wall mark, `wmark(true)`, which the engine draws with its own depth bias.
  pub is_wallmark: bool,
}

impl XRayShaderPassState {
  /// A pass sets depth test and write on unless it says otherwise, which is what a fresh state is.
  const DEFAULT: Self = Self {
    alpha_reference: None,
    blend_destination: None,
    blend_source: None,
    is_alpha_tested: false,
    is_blended: false,
    is_depth_tested: true,
    is_depth_written: true,
    is_wallmark: false,
  };

  /// Reads the state off the calls chained onto one `shader:begin`.
  pub fn of(call: &XRayLuaMethodCall) -> Self {
    let mut state: Self = Self::DEFAULT;

    for chained in call.chained() {
      match chained.method() {
        "blend" => state.read_blend(chained),
        "aref" => state.read_alpha_reference(chained),
        "zb" => state.read_depth(chained),
        "wmark" => state.is_wallmark = Self::is_switched_on(chained),
        _ => {}
      }
    }

    state
  }

  /// Whether the call's first argument is the literal `true`, which is how every one of these switches is written.
  fn is_switched_on(chained: &XRayLuaChainedCall) -> bool {
    chained.argument(0).is_some_and(XRayLuaValue::is_true)
  }

  fn read_blend(&mut self, chained: &XRayLuaChainedCall) {
    self.is_blended = Self::is_switched_on(chained);
    self.blend_source = Self::factor(chained.argument(1));
    self.blend_destination = Self::factor(chained.argument(2));
  }

  fn read_alpha_reference(&mut self, chained: &XRayLuaChainedCall) {
    self.is_alpha_tested = Self::is_switched_on(chained);
    self.alpha_reference = chained
      .argument(1)
      .and_then(XRayLuaValue::as_number)
      .map(|reference| reference.clamp(0.0, f64::from(u8::MAX)) as u8);
  }

  fn read_depth(&mut self, chained: &XRayLuaChainedCall) {
    self.is_depth_tested = Self::is_switched_on(chained);
    self.is_depth_written = chained.argument(1).is_some_and(XRayLuaValue::is_true);
  }

  fn factor(value: Option<&XRayLuaValue>) -> Option<XRayShaderBlendFactor> {
    value
      .and_then(XRayLuaValue::as_name)
      .and_then(XRayShaderBlendFactor::of)
  }
}
