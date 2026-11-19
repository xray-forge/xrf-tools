use std::fs;
use std::path::PathBuf;

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_output::OutputOptions;
use xrf_report::{Finding, Status};
use xrf_test_utils::GeneratedResources;

use crate::commands::archive::list::ArchiveEntrySelection;
use crate::commands::archive::pack::command::PackCommand;
use crate::commands::archive::verify::command::VerifyCommand;
use crate::core::command::CommandResult;
use crate::core::command::command_testing::run_command;

/// A one-file set packed under its own case root, read, and then its volume removed, so the set no longer opens.
fn vanished_set(name: &str) -> CommandResult<(PathBuf, ArchiveProject)> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("archive_verify/{name}"))?;
  let source: PathBuf = root.join("gamedata");
  let destination: PathBuf = root.join("packed");

  fs::create_dir_all(source.join("configs"))?;
  fs::write(source.join("configs").join("system.ltx"), "[section]\nvalue = 1\n")?;

  run_command(
    &PackCommand,
    &[
      String::from("pack"),
      source.display().to_string(),
      String::from("--dest"),
      destination.display().to_string(),
      String::from("--silent"),
    ],
  )?;

  let project: ArchiveProject = ArchiveProject::new(&destination)?;

  fs::remove_file(destination.join("gamedata.db"))?;

  Ok((root, project))
}

#[test]
fn fails_a_set_that_does_not_open_with_entries() -> CommandResult {
  let (root, project) = vanished_set("vanished-with-entries")?;
  let entries: Vec<&ArchiveFileDescriptor> = ArchiveEntrySelection::Files.select(&project);

  assert!(!entries.is_empty());

  let (status, findings): (Status, Vec<Finding>) = VerifyCommand::judge(&project, &entries, &OutputOptions::default());

  assert_eq!(status, Status::Error);
  assert_eq!(findings.len(), 1);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn fails_a_set_that_does_not_open_with_no_entries() -> CommandResult {
  let (root, project) = vanished_set("vanished-without-entries")?;
  let (status, findings): (Status, Vec<Finding>) = VerifyCommand::judge(&project, &[], &OutputOptions::default());

  assert_eq!(status, Status::Error);
  assert_eq!(findings.len(), 1);

  fs::remove_dir_all(root)?;

  Ok(())
}
