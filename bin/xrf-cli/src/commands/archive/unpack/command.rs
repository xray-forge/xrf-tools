use std::env;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_job::{JobHandle, LoggingSink};
use xrf_output::OutputOptions;
use xrf_pack::{ArchiveUnpackOptions, ArchiveUnpackResult, ArchiveUnpacker};
use xrf_utils::format_path;

use super::report::ArchiveUnpackDryReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// How often an unpack says where it has got to.
///
/// Coarse next to what a window would use: these are log lines in a terminal, and a run reporting ten times a second
/// would bury whatever it said before it.
const PROGRESS_INTERVAL: Duration = Duration::from_secs(2);

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
          .help("Count of parallel threads for unpack, defaulting to the host's available parallelism")
          .long("parallel")
          .value_parser(value_parser!(NonZeroUsize)),
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

    // Resolved here rather than by clap: the default is what this host can actually run, which is not a string the
    // parser could carry.
    let parallel: NonZeroUsize = matches
      .get_one::<NonZeroUsize>("parallel")
      .copied()
      .unwrap_or_else(ArchiveUnpacker::get_default_concurrency);

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
          &format_path(path).to_string(),
          &format_path(&destination).to_string(),
        )
      })?;
    } else {
      let result: ArchiveUnpackResult = ArchiveUnpacker::unpack_opt(
        &archive_project,
        &destination,
        ArchiveUnpackOptions::default()
          .with_concurrency(parallel)
          .with_job(JobHandle::with_interval(Arc::new(LoggingSink), PROGRESS_INTERVAL)),
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
  use std::num::NonZeroUsize;

  use crate::core::generic_command::GenericCommand;

  use super::UnpackCommand;

  fn parses(parallel: &str) -> bool {
    UnpackCommand
      .init()
      .try_get_matches_from(["unpack", "--path", "archive.db", "--parallel", parallel])
      .is_ok()
  }

  /// Zero threads once reached a bounded join set as zero permits, where no task acquired one and the command waited
  /// forever on an archive holding a single file. It is still a usage error rejected before any archive is read, and
  /// the reason has only got quieter: Rayon reads `num_threads(0)` as "decide for me", so a zero that got through would
  /// silently run one worker per core instead of the bound the user asked for.
  #[test]
  fn rejects_zero_parallelism() {
    assert!(!parses("0"));
  }

  #[test]
  fn accepts_the_smallest_usable_parallelism() {
    assert!(parses("1"));
  }

  /// The flag has no clap default any more, because the default is the host's parallelism rather than a fixed number.
  /// Omitting it has to stay a valid invocation, resolved after parsing.
  #[test]
  fn accepts_an_omitted_parallelism() {
    assert!(
      UnpackCommand
        .init()
        .try_get_matches_from(["unpack", "--path", "archive.db"])
        .is_ok_and(|matches| matches.get_one::<NonZeroUsize>("parallel").is_none())
    );
  }
}
