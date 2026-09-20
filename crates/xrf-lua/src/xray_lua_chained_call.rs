use crate::xray_lua_value::XRayLuaValue;

/// One call chained onto another's result: the `:blend(...)` of `shader:begin(...):blend(...)`.
#[derive(Clone, Debug, PartialEq)]
pub struct XRayLuaChainedCall {
  arguments: Vec<XRayLuaValue>,
  method: String,
}

impl XRayLuaChainedCall {
  pub(crate) fn from_parts(method: String, arguments: Vec<XRayLuaValue>) -> Self {
    Self { arguments, method }
  }

  pub fn method(&self) -> &str {
    &self.method
  }

  pub fn arguments(&self) -> &[XRayLuaValue] {
    &self.arguments
  }

  /// One argument by position, for a caller reading a call whose shape it knows.
  pub fn argument(&self, at: usize) -> Option<&XRayLuaValue> {
    self.arguments.get(at)
  }
}
