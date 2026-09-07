/// One entry of a token property's list: the value it selects and the label the editor shows for it.
///
/// `xrP_TOKEN::Item` (`xrEngine/Properties.h`). The list is authoring data - `B_MODEL`'s tessellation modes are the
/// only one in the corpus - and it is read because the payload's width depends on how many entries follow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderBlenderToken {
  pub id: u32,
  pub label: String,
}
