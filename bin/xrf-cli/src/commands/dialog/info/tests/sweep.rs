//! What a sweep counts, and how it answers.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;
use xrf_report::Status;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::commands::dialog::info::command::InfoCommand;
use crate::commands::dialog::info::dialog_sweep::{DialogSweep, DialogSweepResult, sum_findings};
use crate::core::command_error::CommandError;
use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::CommandResult;

/// A loose temp root declares no installation, so name the mode rather than letting Auto search upward.
fn roots(root: &Path) -> XrayRoots {
  XrayRoots::one(root.display().to_string(), XrayMountMode::Directory)
}

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-cli-info-dialog-sweep-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

/// Run the command over a path, as the process would.
fn run(path: &Path, extra: &[&str]) -> CommandResult {
  run_for_result(path, extra).map(|_| ())
}

/// Run the command and read back the structured result it deposited.
fn run_for_result(path: &Path, extra: &[&str]) -> CommandResult<Option<Value>> {
  let command: InfoCommand = InfoCommand;
  let mut arguments: Vec<String> = vec![
    String::from("info"),
    String::from("--path"),
    path.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  run_command_for_result(&command, &arguments)
}

#[test]
fn counts_what_the_files_hold() -> XrfResult {
  let root: PathBuf = create_root("census")?;

  fs::write(
    root.join("dialogs.xml"),
    r#"<game_dialogs>
      <dialog id="with_phrases" priority="-5">
        <precondition>dialogs.npc_stalker</precondition>
        <phrase_list>
          <phrase id="0"><text>a</text><next>1</next><next>2</next></phrase>
          <phrase id="1"><text>b</text><is_final>1</is_final></phrase>
          <phrase id="2"><script_text>dialog_manager.create_bye_phrase</script_text></phrase>
        </phrase_list>
      </dialog>
      <dialog id="script_built">
        <init_func>dialog_manager.init_new_dialog</init_func>
      </dialog>
      <dialog id="bare_phrase"><phrase id="0"/></dialog>
    </game_dialogs>"#,
  )?;

  let result: DialogSweepResult = DialogSweep::new(&roots(&root), None).run()?;
  let census = &result.census;

  assert_eq!(census.files, 1);
  assert_eq!(census.dialogs, 3);
  assert_eq!(census.phrases, 4);
  assert_eq!(census.links, 2);
  assert_eq!(census.final_phrases, 1);
  assert_eq!(census.dialogs_with_priority, 1);
  assert_eq!(census.dialogs_without_phrases, 1);
  assert_eq!(census.phrases_outside_phrase_list, 1);
  // The `script_text` phrase and the bare one.
  assert_eq!(census.phrases_without_text, 2);
  assert_eq!(census.largest_dialog_id.as_deref(), Some("with_phrases"));
  assert_eq!(census.largest_dialog_phrases, 3);
  assert_eq!(census.encodings.get("windows-1251"), Some(&1));
  assert_eq!(census.dialog_elements.get("precondition"), Some(&1));
  assert_eq!(census.dialog_elements.get("init_func"), Some(&1));
  assert_eq!(census.phrase_elements.get("next"), Some(&2));
  assert_eq!(census.phrase_elements.get("text"), Some(&2));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reports_an_off_schema_element_as_a_schema_finding() -> XrfResult {
  let root: PathBuf = create_root("schema-finding")?;

  fs::write(
    root.join("dialogs.xml"),
    r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><go_back>1</go_back></phrase></phrase_list></dialog></game_dialogs>"#,
  )?;

  let result: DialogSweepResult = DialogSweep::new(&roots(&root), None).run()?;

  assert_eq!(sum_findings(&result.report), 1);
  assert_eq!(result.report.status(), Status::Failed);

  let schema = result
    .report
    .checks()
    .iter()
    .find(|check| check.id().as_str() == "dialog.schema")
    .expect("the schema check should be reported");

  assert_eq!(schema.status(), Status::Failed);
  assert_eq!(schema.findings()[0].rule_id().as_str(), "dialog.unknown-element");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reports_an_unparsable_file_without_stopping_the_sweep() -> XrfResult {
  let root: PathBuf = create_root("unreadable")?;

  fs::write(root.join("dialogs_broken.xml"), "<game_dialogs><dialog id=\"d\">")?;
  fs::write(
    root.join("dialogs_good.xml"),
    r#"<game_dialogs><dialog id="ok"/></game_dialogs>"#,
  )?;

  let result: DialogSweepResult = DialogSweep::new(&roots(&root), None).run()?;

  assert_eq!(result.census.files, 2);
  assert_eq!(result.census.unreadable_files, 1);
  // The readable file was still read, which is the whole point of sweeping rather than opening.
  assert_eq!(result.census.dialogs, 1);

  let read = result
    .report
    .checks()
    .iter()
    .find(|check| check.id().as_str() == "dialog.read")
    .expect("the read check should be reported");

  assert_eq!(read.status(), Status::Failed);
  assert_eq!(read.findings()[0].rule_id().as_str(), "dialog.unreadable");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn succeeds_on_findings_unless_strict_was_asked_for() -> CommandResult {
  let root: PathBuf = create_root("strict")?;

  fs::write(
    root.join("dialogs.xml"),
    r#"<game_dialogs><dialog id="d" weight="3"/></game_dialogs>"#,
  )?;

  // A tally that also fails the build cannot be run casually, so reporting is the default.
  run(&root, &["--source", "directory"])?;

  // `--strict` is the mode that judges, and it answers 3 per the CLI failure contract.
  match run(&root, &["--source", "directory", "--strict"]) {
    Err(CommandError::CheckFailed { findings }) => assert_eq!(findings, 1),
    other => panic!("expected a check failure, got {other:?}"),
  }

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reports_its_checks_when_a_report_was_asked_for() -> CommandResult {
  let root: PathBuf = create_root("report")?;

  fs::write(
    root.join("dialogs.xml"),
    r#"<game_dialogs><dialog id="d"/></game_dialogs>"#,
  )?;

  // Publishing the deposited payload belongs to the composition root; what the command owes is the
  // payload itself, which reaches a caller under the envelope's `result`.
  let result: Option<Value> = run_for_result(&root, &["--source", "directory", "--json"])?;
  let written: String = serde_json::to_string(&result.expect("a report to be deposited"))?;

  assert!(written.contains("dialog.read"));
  assert!(written.contains("dialog.schema"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn deposits_nothing_when_no_report_was_asked_for() -> CommandResult {
  let root: PathBuf = create_root("no-report")?;

  fs::write(
    root.join("dialogs.xml"),
    r#"<game_dialogs><dialog id="d"/></game_dialogs>"#,
  )?;

  assert_eq!(run_for_result(&root, &["--source", "directory"])?, None);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reads_files_in_a_stable_order() -> XrfResult {
  // A report is only comparable across runs and machines if the files entered it the same way.
  let root: PathBuf = create_root("order")?;

  for name in ["dialogs_zaton.xml", "dialogs.xml", "dialogs_jupiter.xml"] {
    fs::write(
      root.join(name),
      format!(r#"<game_dialogs><dialog id="{name}" weight="1"/></game_dialogs>"#),
    )?;
  }

  let first: DialogSweepResult = DialogSweep::new(&roots(&root), None).run()?;
  let second: DialogSweepResult = DialogSweep::new(&roots(&root), None).run()?;

  let subjects = |result: &DialogSweepResult| -> Vec<String> {
    result
      .report
      .checks()
      .iter()
      .flat_map(|check| check.findings())
      .filter_map(|finding| finding.subject().map(str::to_owned))
      .collect()
  };

  assert_eq!(subjects(&first), subjects(&second));
  assert_eq!(subjects(&first).len(), 3);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_path_that_does_not_exist() -> XrfResult {
  // A typo must not report success, which is the same guard `ogf patch-texture-refs` carries. A missing
  // path now mounts an empty roots rather than failing outright, so the assertion is on the class of
  // failure rather than on wording the mount layer owns.
  let missing: PathBuf = std::env::temp_dir().join("xrf-cli-info-dialog-missing-root");

  match run(&missing, &["--source", "directory"]) {
    Err(error @ CommandError::Execution(_)) => assert_eq!(error.exit_code(), 1),
    other => panic!("expected an execution failure, got {other:?}"),
  }

  Ok(())
}

#[test]
fn refuses_a_directory_holding_no_dialogs() -> XrfResult {
  let root: PathBuf = create_root("no-subjects")?;

  fs::write(root.join("info_zaton.xml"), "<game_information_portions/>")?;

  // Skipped is not a verdict, so it is an execution failure even without `--strict`.
  match run(&root, &["--source", "directory"]) {
    Err(CommandError::Execution(error)) => assert!(error.to_string().contains("No dialog files were read")),
    other => panic!("expected an execution failure, got {other:?}"),
  }

  fs::remove_dir_all(root)?;

  Ok(())
}
