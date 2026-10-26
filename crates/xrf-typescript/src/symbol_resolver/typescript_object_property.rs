/// One property of a resolved object-literal symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScriptObjectProperty {
  pub name: String,
  pub type_name: String,
}
