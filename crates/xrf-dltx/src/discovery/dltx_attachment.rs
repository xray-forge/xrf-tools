/// One mod file patching a base config, with the load rank the engine gives it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltxAttachment {
  /// File name, in the base config's own directory.
  pub name: String,
  /// Load rank. Negative, and lower wins: the alphabetically last mod file has the lowest.
  pub depth: i32,
}
