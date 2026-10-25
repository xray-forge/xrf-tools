/// One plugin's modules as the merged signatures import them: its typed module, and the raw and bulk ones beside it
/// where it has any.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PluginModules<'a> {
  pub name: &'a str,
  pub has_raw: bool,
  pub has_bulk: bool,
}
