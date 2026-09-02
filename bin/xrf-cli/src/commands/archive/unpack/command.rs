use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveUnpackOptions, ArchiveUnpackResult, ArchiveUnpacker};
use xrf_utils::format_path;

use super::report::ArchiveUnpackDryReport;
use crate::core::command_context::CommandContext;
use crate::core::execution::ExecutionArguments;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

#[derive(Default)]
pub struct UnpackCommand;

impl GenericCommand for UnpackCommand {
  fn operation(&self) -> &'static str {
    "unpack"
  }

  /// Create command to unpack archive.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to unpack provided *.db into separate files")
      .arg(
        Arg::new("path")
          .help("Path to *.db file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to folder for exporting")
          .short('d')
          .long("dest")
          .default_value("unpacked")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dry")
          .help("Run in dry mode without actually unpacking to disk")
          .long("dry")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .with_jobs()
  }

  /// Unpack xray engine database archive.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: PathBuf = xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>("path")
        .expect("Expected valid path to be provided"),
    )?;
    let destination: PathBuf = xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>("dest")
        .expect("Expected valid output path to be provided"),
    )?;

    let is_dry: bool = matches.get_flag("dry");

    let output: OutputOptions = context.get_output().clone();

    if is_dry {
      xrf_output::info!(output, "Unpack in dry mode");
    }

    xrf_output::info!(output, "Unpack source: {}", format_path(&path));
    xrf_output::info!(output, "Unpack destination: {}", format_path(&destination));

    let archive_project: Box<ArchiveProject> = Box::new(ArchiveProject::new(&path)?);

    let (compressed_size, real_size): (String, String) =
      xrf_utils::format_bytes_pair(archive_project.get_compressed_size(), archive_project.get_real_size());

    xrf_output::info!(
      output,
      "Summary: {} archive(s), {} file(s), {} compressed, {} real",
      archive_project.archives.len(),
      archive_project.files.len(),
      compressed_size,
      real_size,
    );

    if is_dry {
      // A dry run's whole result is what it would have written, which is the summary it just printed.
      context.set_result(|| {
        ArchiveUnpackDryReport::new(
          &archive_project,
          &format_path(&path).to_string(),
          &format_path(&destination).to_string(),
        )
      })?;
    } else {
      let result: ArchiveUnpackResult = ArchiveUnpacker::unpack_opt(
        &archive_project,
        &destination,
        ArchiveUnpackOptions::default().with_job(new_logging_job()),
      )?;

      xrf_output::success!(
        output,
        "Unpacked archive in {} (preparation {}, unpack {})",
        xrf_utils::format_duration(result.duration),
        xrf_utils::format_duration(result.prepare_duration),
        xrf_utils::format_duration(result.unpack_duration),
      );

      context.set_result(|| &result)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_job::ExecutionRequest;

  use super::UnpackCommand;
  use crate::core::execution::requested_execution;
  use crate::core::generic_command::GenericCommand;

  fn parse(arguments: &[&str]) -> Result<clap::ArgMatches, clap::Error> {
    UnpackCommand.init().try_get_matches_from(arguments)
  }

  /// `--parallel` is gone rather than aliased.
  ///
  /// It named a count of unpack threads, which is now one spelling of one question the whole CLI answers the same way.
  /// An alias would have kept a second name for a plan the report can only print under one of them.
  #[test]
  fn no_longer_answers_to_the_flag_it_replaced() {
    assert!(parse(&["unpack", "--path", "archive.db", "--parallel", "4"]).is_err());
  }

  #[test]
  fn takes_the_width_every_other_command_takes() {
    let matches = parse(&["unpack", "--path", "archive.db", "-j", "4"]).expect("the arguments parse");

    assert_eq!(
      requested_execution(&matches),
      Some(ExecutionRequest::Workers(4.try_into().expect("four is not zero")))
    );
  }

  /// Omitting it stays valid and means what it always meant here: whatever the host offers.
  #[test]
  fn unpacks_at_the_host_width_when_nobody_says_otherwise() {
    let matches = parse(&["unpack", "--path", "archive.db"]).expect("the arguments parse");

    assert_eq!(requested_execution(&matches), Some(ExecutionRequest::Auto));
  }
}
