/// One entry of a token property's list: the value it selects and the label the editor shows for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderBlenderToken {
  pub id: u32,
  pub label: String,
}
