/// One call a script passes literal strings to: the function it calls, as far as its spelling names one, and those
/// strings in argument order.
///
/// `db.actor:give_info_portion("name")` calls `give_info_portion`, `xr_effects.give_info(...)` calls `give_info`, and
/// `hasInfoPortion("name")` calls `hasInfoPortion`; a call through anything else, such as `handlers[key]("name")`, names
/// no function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XRayLuaStringCall {
  function: Option<String>,
  arguments: Vec<String>,
}

impl XRayLuaStringCall {
  pub(crate) fn from_parts(function: Option<String>, arguments: Vec<String>) -> Self {
    Self { function, arguments }
  }

  /// The name of the function called: a method's, the last name of a dotted path, or a bare name.
  pub fn get_function(&self) -> Option<&str> {
    self.function.as_deref()
  }

  /// The literal strings passed, in argument order, other arguments left out.
  pub fn get_arguments(&self) -> &[String] {
    &self.arguments
  }
}
