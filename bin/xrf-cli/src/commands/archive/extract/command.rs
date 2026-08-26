use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveExtractDirectoryResult, ArchiveExtractResult, ArchiveUnpacker};

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct ExtractCommand;

impl GenericCommand for ExtractCommand {
  fn operation(&self) -> &'static str {
    "extract"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Extract one archive file or directory without unpacking the complete set")
      .arg(
        Arg::new("path")
          .help("Path to an archive volume or a directory containing volumes")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
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
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let destination: &PathBuf = matches
      .get_one("dest")
      .expect("Expected an extraction destination to be provided");
    let output: OutputOptions = context.get_output().clone();

    let project: ArchiveProject = ArchiveProject::new(path)?;

    if let Some(file) = matches.get_one::<String>("file") {
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

      let result: ArchiveExtractDirectoryResult = ArchiveUnpacker::extract_directory(&project, directory, destination)?;

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
