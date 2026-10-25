/// One parameter in a resolved TypeScript function signature.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScriptFunctionParameter {
  pub name: String,
  pub optional: bool,
  pub type_name: String,
}
