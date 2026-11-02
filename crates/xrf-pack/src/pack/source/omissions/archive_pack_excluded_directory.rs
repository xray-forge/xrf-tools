/// One directory the rules left out, so a run can say so without the walk having read below it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchivePackExcludedDirectory {
  pub(crate) name: String,
  /// Whether everything below it was pruned as well, or only the directory row itself.
  pub(crate) is_recursive: bool,
}
