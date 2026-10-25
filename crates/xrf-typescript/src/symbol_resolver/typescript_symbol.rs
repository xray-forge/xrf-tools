use super::typescript_function_signature::TypeScriptFunctionSignature;
use super::value_inference::object_type;

/// The TypeScript contract resolved for one source symbol.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeScriptSymbol {
  Callable(TypeScriptFunctionSignature),
  Value(String),
  Object(Vec<(String, String)>),
}

impl TypeScriptSymbol {
  /// Return the canonical value type when this symbol is not callable.
  pub fn value_type(&self) -> Option<String> {
    match self {
      Self::Callable(_) => None,
      Self::Value(type_name) => Some(type_name.clone()),
      Self::Object(properties) => Some(object_type(properties)),
    }
  }

  /// Return a named property type from an object-literal symbol.
  pub fn property_type(&self, property_name: &str) -> Option<String> {
    let Self::Object(properties) = self else {
      return None;
    };

    properties
      .iter()
      .find(|(name, _)| name == property_name)
      .map(|(_, type_name)| type_name.clone())
  }
}
