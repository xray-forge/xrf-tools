use xrf_output::OutputOptions;
use xrf_pack::{ArchivePatchConfig, ArchivePatchPublication, ArchivePatchResult};
use xrf_utils::format_path;

/// Prints the comparison roots and, for publishing runs, the destination.
pub(crate) fn describe_inputs(output: &OutputOptions, config: &ArchivePatchConfig, is_dry_run: bool) {
  xrf_output::info!(output, "Patch input: {}", format_path(&config.input));

  match config.target.as_deref() {
    Some(target) => xrf_output::info!(output, "Patch target: {}", format_path(target)),
    None => xrf_output::info!(output, "Patch target: loose gamedata of the input"),
  }

  if !is_dry_run {
    xrf_output::info!(output, "Patch destination: {}", format_path(&config.destination));
  }
}

/// Prints the publication outcome, counts, sizes, timings, and a warning for entries the patch cannot delete.
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

  // Always said, including on a comparison: the carried bytes are known before anything is written, and "how big is
  // this patch going to be" is the question a preview exists to answer. What the volumes weigh is only added once
  // they exist, because compression and payload sharing decide it.
  match result.publication.get_published() {
    Some(published) => {
      let (size_carried, size_written): (String, String) =
        xrf_utils::format_bytes_pair(result.size_carried, published.size_written);

      xrf_output::info!(output, "Size: {size_carried} carried, {size_written} written");
    }
    None => xrf_output::info!(
      output,
      "Size: {} to carry",
      xrf_utils::format_bytes(result.size_carried)
    ),
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
