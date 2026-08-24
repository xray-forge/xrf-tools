use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct ListCommand;

/// The name-table records to include in a listing.
#[derive(Clone, Copy)]
pub(crate) enum ArchiveEntrySelection {
  All,
  Files,
  Directories,
}

impl GenericCommand for ListCommand {
  fn operation(&self) -> &'static str {
    "list"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("List the merged entries in an X-Ray archive volume or volume set")
      .arg(
        Arg::new("path")
          .help("Path to an archive volume or a directory containing volumes")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("files")
          .help("List files only, excluding directory records")
          .long("files")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("directories")
          .help("List directory records only, excluding files")
          .long("directories")
          .action(ArgAction::SetTrue),
      )
      .group(ArgGroup::new("entry-kind").args(["files", "directories"]))
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Include stored size, unpacked size, and source volume")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let entries: Vec<&ArchiveFileDescriptor> = Self::entries(&project, Self::selection(matches));

    for entry in &entries {
      if matches.get_flag("verbose") {
        let (compressed, unpacked): (String, String) =
          xrf_utils::format_bytes_pair(u64::from(entry.size_compressed), u64::from(entry.size_real));

        xrf_output::info!(
          output,
          "{} [{compressed} stored, {unpacked} unpacked, {}]",
          entry.name,
          entry.source.display(),
        );
      } else {
        xrf_output::info!(output, "{}", entry.name);
      }
    }

    xrf_output::success!(
      output,
      "Listed {} entr{} in {}",
      entries.len(),
      if entries.len() == 1 { "y" } else { "ies" },
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

impl ListCommand {
  pub(crate) fn selection(matches: &ArgMatches) -> ArchiveEntrySelection {
    if matches.get_flag("files") {
      ArchiveEntrySelection::Files
    } else if matches.get_flag("directories") {
      ArchiveEntrySelection::Directories
    } else {
      ArchiveEntrySelection::All
    }
  }

  pub(crate) fn entries(project: &ArchiveProject, selection: ArchiveEntrySelection) -> Vec<&ArchiveFileDescriptor> {
    let mut entries: Vec<&ArchiveFileDescriptor> = project
      .files
      .values()
      .filter(|entry| match selection {
        ArchiveEntrySelection::All => true,
        ArchiveEntrySelection::Files => !entry.is_directory,
        ArchiveEntrySelection::Directories => entry.is_directory,
      })
      .collect();

    entries.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    entries
  }
}

#[cfg(test)]
mod tests {
  use crate::core::generic_command::GenericCommand;

  use super::ListCommand;

  #[test]
  fn rejects_conflicting_entry_kind_filters() {
    assert!(
      ListCommand
        .init()
        .try_get_matches_from(["list", "--path", "archive.db", "--files", "--directories"])
        .is_err()
    );
  }
}
