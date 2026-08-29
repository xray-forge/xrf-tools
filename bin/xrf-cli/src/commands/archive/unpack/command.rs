use std::env;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use tokio::runtime::Runtime;
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveUnpackResult, ArchiveUnpacker};
use xrf_utils::format_path;

use super::report::ArchiveUnpackDryReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

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
  }

  /// Unpack xray engine database archive.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
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

    let output: OutputOptions = context.get_output().clone();

    if is_dry {
      xrf_output::info!(output, "Unpack in dry mode");
    }

    xrf_output::info!(output, "Unpack source: {}", format_path(path));
    xrf_output::info!(output, "Unpack destination: {}", format_path(&destination));

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

    if is_dry {
      // A dry run's whole result is what it would have written, which is the summary it just printed.
      context.set_result(|| {
        ArchiveUnpackDryReport::new(
          &archive_project,
          &xrf_utils::to_portable_path_string(path),
          &xrf_utils::to_portable_path_string(&destination),
        )
      })?;
    } else {
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

      context.set_result(|| &result)?;
    }

    Ok(())
  }
}
