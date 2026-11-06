/// Where a compiled OGF says it came from, its description's own lines: the source file, and what built it when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectCompileSource {
  /// The `.object` it was compiled from, as the tool was given it.
  pub source_file: String,
  /// The tool and version that compiled it.
  pub convertor: String,
  /// Seconds since the Unix epoch.
  pub built_at: u64,
}
