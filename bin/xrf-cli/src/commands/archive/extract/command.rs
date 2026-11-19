use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveExtractResult, ArchiveUnpacker};
use xrf_utils::OutputFiles;

use crate::core::arguments::{InputArgument, JobsArgument, LinkPolicyArgument};
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub(crate) struct ExtractCommand;

impl GenericCommand for ExtractCommand {
  fn operation(&self) -> &'static str {
    "extract"
  }

  fn input(&self) -> CommandInput {
    CommandInput::Path("Path to an archive volume or a directory containing volumes")
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Extract one archive file or directory without unpacking the complete set")
      .arg(
        Arg::new("file")
          .help("Exact logical path of one archive file; --dest is the output file")
          .long("file")
          .required_unless_present("directory")
          .conflicts_with("directory")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("directory")
          .help("Logical directory to extract; --dest receives that directory's contents")
          .long("directory")
          .required_unless_present("file")
          .conflicts_with("file")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("dest")
          .help("Output file for --file, or output directory for --directory")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help(
            "Overwrite the output file, or files already in the output directory, names compared without case; \
             without it, one there is refused before any is written",
          )
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      );

    LinkPolicyArgument::declare(JobsArgument::declare(command))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = InputArgument::of(matches);
    let destination: &PathBuf = matches
      .get_one("dest")
      .expect("Expected an extraction destination to be provided");
    let output: OutputOptions = context.get_output().clone();
    let is_forced: bool = matches.get_flag("force");

    let project: ArchiveProject = ArchiveProject::new(path)?;

    if let Some(file) = matches.get_one::<String>("file") {
      OutputFiles::of_path(destination).assert_free(is_forced)?;

      let result: ArchiveExtractResult = ArchiveUnpacker::extract_file(&project, file, destination)?;

      xrf_output::success!(
        output,
        "Extracted {} ({}) in {}",
        result.name,
        xrf_utils::format_bytes(result.size),
        xrf_utils::format_duration(started_at.elapsed())
      );

      context.set_result(|| &result)?;
    } else {
      let directory: &String = matches
        .get_one("directory")
        .expect("Expected a file or directory to be selected");

      let result: ArchiveExtractDirectoryResult = ArchiveUnpacker::extract_directory_opt(
        &project,
        directory,
        destination,
        ArchiveExtractOptions::default()
          .with_job(CliJob::for_output(&output))
          .with_link_policy(LinkPolicyArgument::of(matches))
          .with_forced(is_forced),
      )?;

      xrf_output::success!(
        output,
        "Extracted {} file(s) from {} ({} total) in {}",
        result.extracted_count,
        result.prefix,
        xrf_utils::format_bytes(result.size),
        xrf_utils::format_duration(started_at.elapsed()),
      );

      context.set_result(|| &result)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_job::ExecutionRequest;
  use xrf_utils::HostLinkPolicy;

  use super::ExtractCommand;
  use crate::core::arguments::{JobsArgument, LinkPolicyArgument};
  use crate::core::command::GenericCommand;

  fn parse(arguments: &[&str]) -> Result<clap::ArgMatches, clap::Error> {
    ExtractCommand.init().try_get_matches_from(arguments)
  }

  #[test]
  fn takes_the_width_every_other_command_takes() {
    let matches = parse(&[
      "extract",
      "archive.db",
      "--directory",
      "scripts",
      "-d",
      "out",
      "-j",
      "1",
    ])
    .expect("the arguments parse");

    assert_eq!(
      JobsArgument::of(&matches),
      Some(ExecutionRequest::Workers(1.try_into().expect("one is not zero")))
    );
  }

  #[test]
  fn refuses_links_below_the_destination_only_when_asked() {
    let following =
      parse(&["extract", "archive.db", "--directory", "scripts", "-d", "out"]).expect("the arguments parse");
    let refusing = parse(&[
      "extract",
      "archive.db",
      "--directory",
      "scripts",
      "-d",
      "out",
      "--refuse-links",
    ])
    .expect("the arguments parse");

    assert_eq!(LinkPolicyArgument::of(&following), HostLinkPolicy::Follow);
    assert_eq!(LinkPolicyArgument::of(&refusing), HostLinkPolicy::Refuse);
  }

  #[test]
  fn extracts_at_the_host_width_when_nobody_says_otherwise() {
    let matches =
      parse(&["extract", "archive.db", "--directory", "scripts", "-d", "out"]).expect("the arguments parse");

    assert_eq!(JobsArgument::of(&matches), Some(ExecutionRequest::Auto));
  }
}
