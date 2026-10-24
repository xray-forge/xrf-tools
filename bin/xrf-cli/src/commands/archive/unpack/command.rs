use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveExtractOptions, ArchiveUnpackResult, ArchiveUnpacker};
use xrf_utils::format_path;

use super::report::ArchiveUnpackDryReport;
use crate::core::arguments::{JobsArgument, LinkPolicyArgument};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct UnpackCommand;

impl GenericCommand for UnpackCommand {
  fn operation(&self) -> &'static str {
    "unpack"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Unpack an archive into separate files")
      .arg(
        Arg::new("path")
          .help("Path to the *.db archive")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Folder to unpack into")
          .short('d')
          .long("dest")
          .default_value("unpacked")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dry-run")
          .help("Report what would be unpacked without writing anything")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      );

    LinkPolicyArgument::declare(JobsArgument::declare(command))
  }

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

    let is_dry_run: bool = matches.get_flag("dry-run");

    let output: OutputOptions = context.get_output().clone();

    if is_dry_run {
      xrf_output::info!(output, "Unpack in dry mode");
    }

    xrf_output::info!(output, "Unpack source: {}", format_path(&path));
    xrf_output::info!(output, "Unpack destination: {}", format_path(&destination));

    let archive_project: ArchiveProject = ArchiveProject::new(&path)?;

    let (compressed_size, real_size): (String, String) =
      xrf_utils::format_bytes_pair(archive_project.get_compressed_size(), archive_project.get_real_size());

    xrf_output::info!(
      output,
      "Summary: {} archive(s), {} file(s), {} directory record(s), {} compressed, {} real",
      archive_project.archives.len(),
      archive_project.count_files(),
      archive_project.count_directories(),
      compressed_size,
      real_size,
    );

    if is_dry_run {
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
        ArchiveExtractOptions::default()
          .with_job(CliJob::new_logging())
          .with_link_policy(LinkPolicyArgument::of(matches)),
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
  use crate::core::arguments::JobsArgument;
  use crate::core::command::GenericCommand;

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
      JobsArgument::of(&matches),
      Some(ExecutionRequest::Workers(4.try_into().expect("four is not zero")))
    );
  }

  /// Omitting it stays valid and means what it always meant here: whatever the host offers.
  #[test]
  fn unpacks_at_the_host_width_when_nobody_says_otherwise() {
    let matches = parse(&["unpack", "--path", "archive.db"]).expect("the arguments parse");

    assert_eq!(JobsArgument::of(&matches), Some(ExecutionRequest::Auto));
  }
}
