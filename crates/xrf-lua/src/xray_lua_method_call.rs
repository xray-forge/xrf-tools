use crate::xray_lua_chained_call::XRayLuaChainedCall;
use crate::xray_lua_value::XRayLuaValue;

/// A Lua method call with its receiver, its arguments, whatever is chained onto it, and where it was written.
#[derive(Clone, Debug, PartialEq)]
pub struct XRayLuaMethodCall {
  arguments: Vec<XRayLuaValue>,
  /// The calls chained onto this one's result, in source order.
  chained: Vec<XRayLuaChainedCall>,
  /// The function this call was written in, for a reader that cares which one the engine invokes.
  function: Option<String>,
  line_number: usize,
  method: String,
  receiver: String,
}

impl XRayLuaMethodCall {
  pub(crate) fn from_parts(
    line_number: usize,
    receiver: String,
    method: String,
    arguments: Vec<XRayLuaValue>,
    chained: Vec<XRayLuaChainedCall>,
  ) -> Self {
    Self {
      arguments,
      chained,
      function: None,
      line_number,
      method,
      receiver,
    }
  }

  pub(crate) fn set_function(&mut self, function: Option<String>) {
    self.function = function;
  }

  pub fn line_number(&self) -> usize {
    self.line_number
  }

  pub fn method(&self) -> &str {
    &self.method
  }

  pub fn receiver(&self) -> &str {
    &self.receiver
  }

  pub fn arguments(&self) -> &[XRayLuaValue] {
    &self.arguments
  }

  pub fn chained(&self) -> &[XRayLuaChainedCall] {
    &self.chained
  }

  /// The function this call sits in, or `None` for one written at the top level.
  pub fn function(&self) -> Option<&str> {
    self.function.as_deref()
  }

  /// The first chained call of a name, for a reader looking for one state among the chain.
  pub fn chained_call(&self, method: &str) -> Option<&XRayLuaChainedCall> {
    self.chained.iter().find(|call| call.method() == method)
  }

  /// The arguments, for a call whose arguments are all literal strings.
  pub fn literal_string_arguments(&self) -> Option<Vec<String>> {
    self
      .arguments
      .iter()
      .map(|argument| argument.as_string().map(str::to_owned))
      .collect()
  }
}
