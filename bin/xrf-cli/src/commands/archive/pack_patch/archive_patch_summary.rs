use xrf_output::OutputOptions;
use xrf_pack::{ArchivePatchConfig, ArchivePatchPublication, ArchivePatchResult};
use xrf_utils::format_path;

/// Say what the run was pointed at, before it costs anything to find out it was the wrong pair.
pub(crate) fn describe_inputs(output: &OutputOptions, config: &ArchivePatchConfig, is_dry_run: bool) {
  for base in &config.base {
    xrf_output::info!(output, "Patch base: {}", format_path(base));
  }

  for target in &config.target {
    xrf_output::info!(output, "Patch target: {}", format_path(target));
  }

  if !is_dry_run {
    xrf_output::info!(output, "Patch destination: {}", format_path(&config.destination));
  }
}

/// Say what the comparison found, and what became of it.
///
/// The headline comes off `publication`, so the three outcomes a run can have are three branches rather than a
/// sentence assembled from an empty volume list and a flag. Removals get a line of their own instead of a place in the
/// counts: they are the one class the format cannot carry, and a reader skimming a successful run would otherwise
/// never learn they exist.
pub(crate) fn describe_result(output: &OutputOptions, result: &ArchivePatchResult) {
  describe_headline(output, result);

  if !result.removed.is_empty() {
    xrf_output::warning!(
      output,
      "{} entry(s) the base holds are absent from the target. A '.db' patch cannot express a deletion, so these stay \
       readable from the base; they are listed in the report.",
      result.removed.len()
    );
  }

  xrf_output::info!(
    output,
    "Phases: {} comparing, {} packing",
    xrf_utils::format_duration(result.compare_duration),
    xrf_utils::format_duration(result.pack_duration),
  );

  xrf_output::info!(
    output,
    "Summary: {} added, {} modified, {} unchanged, {} payload(s) read to decide",
    result.added.len(),
    result.modified.len(),
    result.unchanged,
    result.payloads_read,
  );

  if let Some(published) = result.publication.get_published() {
    let (size_source, size_written): (String, String) =
      xrf_utils::format_bytes_pair(published.size_source, published.size_written);

    xrf_output::info!(output, "Size: {size_source} carried, {size_written} written");
  }
}

fn describe_headline(output: &OutputOptions, result: &ArchivePatchResult) {
  match &result.publication {
    ArchivePatchPublication::Compared => xrf_output::success!(
      output,
      "Compared in {}: {} to add, {} to modify, {} unchanged. Nothing written.",
      xrf_utils::format_duration(result.duration),
      result.added.len(),
      result.modified.len(),
      result.unchanged,
    ),
    ArchivePatchPublication::Unnecessary => xrf_output::success!(
      output,
      "Compared in {}: nothing differs, so no patch was written. {} entry(s) match.",
      xrf_utils::format_duration(result.duration),
      result.unchanged,
    ),
    ArchivePatchPublication::Published(published) => {
      for volume in &published.volumes {
        xrf_output::info!(output, "Wrote {}", format_path(volume));
      }

      xrf_output::success!(
        output,
        "Patched {} file(s) into {} volume(s) in {}",
        result.get_carried_count(),
        published.volumes.len(),
        xrf_utils::format_duration(result.duration),
      );
    }
  }
}
