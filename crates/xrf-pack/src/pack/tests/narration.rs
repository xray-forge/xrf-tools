//! What a run says about itself, which is a contract of its own: a transcript is what someone reads when a pack goes
//! wrong, and it is worth nothing if it disagrees with the archive beside it.
//!
//! Asserted against the same results the other checks read, so a line claiming a file was compressed and a summary
//! counting it as stored cannot both pass. What a run says is also all it says: a quiet run says nothing at all.

use std::sync::Arc;

use xrf_output::{OutputOptions, OutputVerbosity, RecordingOutput};

use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory, ArchivePackMode};
use crate::pack::tests::fixtures::{BINARY, CONFIG, create_config};
use crate::pack::{ArchivePackOptions, ArchivePackResult, ArchivePacker};

/// Pack a freshly built tree at `verbosity`, answering the run and every line it rendered.
fn pack_saying(
  scope: &str,
  verbosity: OutputVerbosity,
  files: &[(&str, &[u8])],
  configure: impl FnOnce(&mut ArchivePackConfig),
) -> (ArchivePackResult, Vec<String>) {
  let (mut config, _) = create_config(scope, files);

  configure(&mut config);

  let recorded: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
  let options: ArchivePackOptions =
    ArchivePackOptions::default().with_output(OutputOptions::new(recorded.clone(), verbosity));
  let result: ArchivePackResult = ArchivePacker::pack_opt(&config, options).expect("source tree packs");

  let lines: Vec<String> = recorded
    .list_records()
    .iter()
    .map(|record| record.get_message().to_owned())
    .collect();

  (result, lines)
}

/// The one line beginning with `prefix`, so a failure names what was said instead of only that something was not.
fn line_of<'l>(lines: &'l [String], prefix: &str) -> &'l str {
  let mut matching = lines.iter().filter(|line| line.starts_with(prefix));
  let line: &String = matching
    .next()
    .unwrap_or_else(|| panic!("no line begins with '{prefix}', the run said: {lines:#?}"));

  assert!(
    matching.next().is_none(),
    "'{prefix}' was said more than once: {lines:#?}"
  );

  line
}

#[test]
fn a_run_nobody_is_reading_says_nothing() {
  // The default options, which is what every caller that does not ask for output gets.
  let (result, lines) = pack_saying(
    "a_run_nobody_is_reading_says_nothing",
    OutputVerbosity::Normal,
    &[("configs\\system.ltx", CONFIG), ("readme.txt", b"notes")],
    |_| {},
  );

  assert_eq!(result.files_total, 1, "the run itself is unaffected");
  assert!(lines.is_empty(), "a normal run keeps the transcript to itself");
}

#[test]
fn a_verbose_run_names_its_settings_before_it_does_anything() {
  let (_, lines) = pack_saying(
    "a_verbose_run_names_its_settings_before_it_does_anything",
    OutputVerbosity::Verbose,
    &[("configs\\system.ltx", CONFIG)],
    |config| config.mode = ArchivePackMode::Store,
  );

  // First, because every later line is read against it: why a file was stored, where a volume was cut.
  assert!(
    lines[0].starts_with("Pack mode: store, volume cap "),
    "the settings open the transcript: {lines:#?}"
  );
  assert!(
    lines[0].ends_with(", skip list on, 0 excluded extension(s)"),
    "{}",
    lines[0]
  );
}

#[test]
fn every_placement_is_said_the_way_the_counts_record_it() {
  let (result, lines) = pack_saying(
    "every_placement_is_said_the_way_the_counts_record_it",
    OutputVerbosity::Verbose,
    // Named so the engine-name order the writer places them in reads as the story the lines tell.
    &[
      ("configs\\a_system.ltx", CONFIG),
      ("configs\\b_copy.ltx", CONFIG),
      ("configs\\c_tiny.ltx", b"[a]"),
      ("configs\\d_empty.ltx", b""),
      ("textures\\wall.dds", BINARY),
    ],
    |_| {},
  );

  // One line per entry, each naming the outcome the summary counted it under.
  assert!(line_of(&lines, "Compressed: configs\\a_system.ltx,").contains(" -> "));
  assert_eq!(
    line_of(&lines, "Aliased: configs\\b_copy.ltx"),
    "Aliased: configs\\b_copy.ltx -> configs\\a_system.ltx",
    "an alias names the entry whose payload it points at"
  );
  assert!(line_of(&lines, "Reverted: configs\\c_tiny.ltx,").ends_with("(compression saved nothing)"));
  assert_eq!(
    line_of(&lines, "Empty: configs\\d_empty.ltx"),
    "Empty: configs\\d_empty.ltx"
  );
  assert!(line_of(&lines, "Stored: textures\\wall.dds,").ends_with("B"));

  // The three the transcript tells apart are one heading in the summary, which is what the volume holds.
  assert_eq!(result.files_compressed, 1);
  assert_eq!(result.files_aliased, 1);
  assert_eq!(result.files_stored, 3, "reverted, empty, and stored");
}

#[test]
fn a_rule_that_left_something_out_says_which_rule_it_was() {
  let (result, lines) = pack_saying(
    "a_rule_that_left_something_out_says_which_rule_it_was",
    OutputVerbosity::Verbose,
    &[
      ("configs\\system.ltx", CONFIG),
      ("readme.txt", b"notes"),
      ("textures\\wall.thm", BINARY),
      ("scripts\\dropped\\old.script", CONFIG),
      // Its own payload, so it is placed rather than aliased onto the configuration above it.
      ("shaders\\r1\\clouds.vs", b"float4 main() : COLOR { return 0; }"),
    ],
    |config| {
      config.exclude_extensions = vec![String::from(".thm")];
      config.exclude_directories = vec![
        ArchivePackDirectory {
          path: String::from("scripts\\dropped"),
          is_recursive: true,
        },
        ArchivePackDirectory {
          path: String::from("shaders"),
          is_recursive: false,
        },
      ];
    },
  );

  assert_eq!(
    line_of(&lines, "Skipped: readme.txt"),
    "Skipped: readme.txt (skip list)"
  );
  assert_eq!(
    line_of(&lines, "Skipped: textures\\wall.thm"),
    "Skipped: textures\\wall.thm (excluded extension)"
  );
  assert_eq!(
    line_of(&lines, "Excluded directory: scripts\\dropped"),
    "Excluded directory: scripts\\dropped (recursive rule)",
    "a pruned directory is said even though the walk never yielded it"
  );
  assert_eq!(
    line_of(&lines, "Excluded directory: shaders "),
    "Excluded directory: shaders (shallow rule)",
    "a shallow rule drops the row while its contents still pack"
  );

  assert_eq!(result.files_skipped, 2, "only the skip rules count as skipped");
  // The shallow rule kept the file below it, which is what makes it worth telling from the recursive one.
  assert!(line_of(&lines, "Stored: shaders\\r1\\clouds.vs,").ends_with("B"));
}

/// The settled transcript order, which the CLI snapshots in `xrf-tools-e2e` pin line for line: the settings, then the
/// whole selection as one block, then the volumes and what went into them.
///
/// Grouped rather than interleaved on purpose. It is also why the walk's omissions are collected and ordered instead
/// of said where they are found — see `issues/closed/0104`.
#[test]
fn the_selection_is_one_block_between_the_settings_and_the_first_volume() {
  let (_, lines) = pack_saying(
    "the_selection_is_one_block_between_the_settings_and_the_first_volume",
    OutputVerbosity::Verbose,
    &[
      ("configs\\system.ltx", CONFIG),
      ("readme.txt", b"notes"),
      ("misc\\dropped.ltx", CONFIG),
    ],
    |config| {
      config.exclude_directories = vec![ArchivePackDirectory {
        path: String::from("misc"),
        is_recursive: true,
      }];
    },
  );
  let kinds: Vec<&str> = lines
    .iter()
    .map(|line| line.split(':').next().expect("a leading word"))
    .collect();

  assert_eq!(
    kinds,
    [
      "Pack mode",
      "Directory",
      "Excluded directory",
      "Skipped",
      "Opened volume",
      "Compressed",
      "Closed volume"
    ]
  );
}

#[test]
fn a_volume_is_named_before_it_is_written_and_measured_once_it_is_closed() {
  let (result, lines) = pack_saying(
    "a_volume_is_named_before_it_is_written_and_measured_once_it_is_closed",
    OutputVerbosity::Verbose,
    &[("configs\\system.ltx", CONFIG)],
    |_| {},
  );

  let opened: usize = lines
    .iter()
    .position(|line| line == "Opened volume: packed.db0")
    .expect("the volume is named as it is opened");
  let written: usize = lines
    .iter()
    .position(|line| line.starts_with("Compressed: configs\\system.ltx,"))
    .expect("the entry is said");

  assert!(opened < written, "the volume is named before anything goes into it");
  assert!(
    line_of(&lines, "Closed volume: packed.db0,").ends_with(", 1 entries"),
    "the closed volume reports the files it took: {lines:#?}"
  );

  // Said under the name it was written as; the set is renamed afterwards, and the transcript does not rewrite itself.
  assert_eq!(result.volumes[0].file_name().expect("volume name"), "packed.db");
}

#[test]
fn the_directory_rows_are_said_as_the_archive_will_carry_them() {
  let (_, lines) = pack_saying(
    "the_directory_rows_are_said_as_the_archive_will_carry_them",
    OutputVerbosity::Verbose,
    &[("configs\\weapons\\w_ak74.ltx", CONFIG), ("textures\\wall.dds", BINARY)],
    |_| {},
  );
  let directories: Vec<&String> = lines.iter().filter(|line| line.starts_with("Directory: ")).collect();

  // Registration's rows, folded and ordered, rather than whatever spellings the walk happened to reach.
  assert_eq!(
    directories,
    [
      "Directory: configs",
      "Directory: configs\\weapons",
      "Directory: textures"
    ]
  );
}

#[test]
fn two_runs_over_one_tree_say_the_same_things_in_the_same_order() {
  // The walk is ordered for this: a transcript nobody can diff against yesterday's is worth much less than one
  // anybody can, and only the walk could have said its lines in a host-dependent order.
  let files: [(&str, &[u8]); 4] = [
    ("configs\\system.ltx", CONFIG),
    ("configs\\weapons\\w_ak74.ltx", CONFIG),
    ("readme.txt", b"notes"),
    ("textures\\wall.dds", BINARY),
  ];
  let scope: &str = "two_runs_over_one_tree_say_the_same_things_in_the_same_order";

  let (_, first) = pack_saying(&format!("{scope}/first"), OutputVerbosity::Verbose, &files, |_| {});
  let (_, second) = pack_saying(&format!("{scope}/second"), OutputVerbosity::Verbose, &files, |_| {});

  assert_eq!(first, second);
}
