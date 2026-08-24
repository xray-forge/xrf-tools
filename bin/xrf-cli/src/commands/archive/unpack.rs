use std::env;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use tokio::runtime::Runtime;
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveUnpackResult, ArchiveUnpacker};

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

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
        Arg::new("parallel")
          .help("Count of parallel threads for unpack")
          .long("parallel")
          .default_value("32")
          .value_parser(value_parser!(usize)),
      )
      .arg(
        Arg::new("dry")
          .help("Run in dry mode without actually unpacking to disk")
          .long("dry")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("silent")
          .help("Turn off logging")
          .short('s')
          .long("silent")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Unpack xray engine database archive.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let destination: &PathBuf = matches
      .get_one::<_>("dest")
      .expect("Expected valid output path to be provided");

    let destination: PathBuf = if destination.is_relative() {
      env::current_dir()?.join(destination)
    } else {
      destination.clone()
    };

    let parallel: usize = *matches
      .get_one::<usize>("parallel")
      .expect("Expected valid parallel threads count to be provided");

    let is_dry: bool = matches.get_flag("dry");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    if is_dry {
      xrf_output::info!(output, "Unpack in dry mode");
    }

    xrf_output::info!(output, "Unpack source: {}", path.display());
    xrf_output::info!(output, "Unpack destination: {}", destination.display());

    let archive_project: Box<ArchiveProject> = Box::new(ArchiveProject::new(path)?);

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

    xrf_output::info!(output, "Unpacking files, parallel {parallel}");

    if !is_dry {
      let result: ArchiveUnpackResult = Runtime::new()?.block_on(ArchiveUnpacker::unpack_parallel(
        &archive_project,
        &destination,
        parallel,
      ))?;

      xrf_output::success!(
        output,
        "Unpacked archive in {} (preparation {}, unpack {})",
        xrf_utils::format_duration(result.duration),
        xrf_utils::format_duration(result.prepare_duration),
        xrf_utils::format_duration(result.unpack_duration),
      );
    }

    Ok(())
  }
}
