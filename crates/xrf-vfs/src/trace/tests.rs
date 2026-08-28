use crate::trace::{XrayReadTrace, XrayReadTraceEntry, XrayReadTraceSummary};

#[test]
fn records_nothing_before_a_read() {
  let trace: XrayReadTrace = XrayReadTrace::default();

  assert!(trace.is_empty());
  assert_eq!(trace.len(), 0);
  assert_eq!(trace.get("meshes\\a.ogf"), None);
}

#[test]
fn counts_every_read_of_one_path() {
  let trace: XrayReadTrace = XrayReadTrace::default();

  trace.record("meshes\\a.ogf", 100);
  trace.record("meshes\\a.ogf", 100);
  trace.record("meshes\\a.ogf", 100);

  assert_eq!(trace.len(), 1);
  assert_eq!(
    trace.get("meshes\\a.ogf"),
    Some(XrayReadTraceEntry {
      reads: 3,
      bytes: 300,
      size: 100,
    })
  );
}

/// Unique volume is a sum of per-read sizes rather than `bytes / reads`, so a path whose length changed
/// between reads still reports what one pass would cost.
#[test]
fn tracks_the_latest_size_separately_from_the_total() {
  let trace: XrayReadTrace = XrayReadTrace::default();

  trace.record("configs\\system.ltx", 100);
  trace.record("configs\\system.ltx", 250);

  let summary: XrayReadTraceSummary = trace.get_summary(10);

  assert_eq!(summary.bytes, 350);
  assert_eq!(summary.unique_bytes, 250);
}

#[test]
fn totals_reads_against_unique_paths() {
  let trace: XrayReadTrace = XrayReadTrace::default();

  trace.record("anims\\shared.omf", 1000);
  trace.record("anims\\shared.omf", 1000);
  trace.record("anims\\shared.omf", 1000);
  trace.record("meshes\\a.ogf", 40);

  let summary: XrayReadTraceSummary = trace.get_summary(10);

  assert_eq!(summary.paths, 2);
  assert_eq!(summary.reads, 4);
  assert_eq!(summary.bytes, 3040);
  assert_eq!(summary.unique_bytes, 1040);
}

#[test]
fn names_the_most_read_paths_first() {
  let trace: XrayReadTrace = XrayReadTrace::default();

  trace.record("meshes\\a.ogf", 40);
  trace.record("anims\\hot.omf", 1000);
  trace.record("anims\\hot.omf", 1000);
  trace.record("anims\\warm.omf", 500);
  trace.record("anims\\warm.omf", 500);

  let summary: XrayReadTraceSummary = trace.get_summary(10);
  let ranked: Vec<&str> = summary.hottest.iter().map(|entry| entry.path.as_str()).collect();

  // Equal read counts order by bytes, so the more expensive bank of the two comes first.
  assert_eq!(ranked, vec!["anims\\hot.omf", "anims\\warm.omf", "meshes\\a.ogf"]);
  assert_eq!(summary.hottest[0].reads, 2);
  assert_eq!(summary.hottest[0].bytes, 2000);
}

/// A sweep touches tens of thousands of paths, so the named list is a slice and `paths` says so.
#[test]
fn caps_the_named_list_without_hiding_the_total() {
  let trace: XrayReadTrace = XrayReadTrace::default();

  for index in 0..10 {
    trace.record(&format!("meshes\\{index}.ogf"), 10);
  }

  let summary: XrayReadTraceSummary = trace.get_summary(3);

  assert_eq!(summary.paths, 10);
  assert_eq!(summary.hottest.len(), 3);
}
