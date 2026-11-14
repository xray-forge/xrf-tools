use serde::Serialize;
use xrf_output::OutputOptions;
use xrf_spawn::LevelSpawnExport;

/// What a format says about one export beyond the folder it wrote, which `unpack` and `re-unpack` print and deposit
/// beside the conversion: nothing for most formats.
pub(crate) trait PackedFileUnpacked: Serialize {
  /// Prints what the export wrote, in the command's output.
  fn print(&self, output: &OutputOptions);
}

impl PackedFileUnpacked for () {
  fn print(&self, _: &OutputOptions) {}
}

/// A level's spawn list says how many objects it wrote typed and how many verbatim, and why; a root spawn says nothing.
impl PackedFileUnpacked for Option<LevelSpawnExport> {
  fn print(&self, output: &OutputOptions) {
    let Some(export) = self else {
      return;
    };

    xrf_output::info!(
      output,
      "Wrote {} objects typed and {} verbatim",
      export.typed_objects,
      export.sum_verbatim_objects()
    );

    for group in &export.verbatim_objects {
      xrf_output::warning!(output, "  {} written verbatim: {}", group.objects, group.reason);
    }
  }
}
