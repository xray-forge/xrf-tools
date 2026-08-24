use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{OmfFile, OmfMotionsProcessor, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct FilterMotionsCommand;

impl GenericCommand for FilterMotionsCommand {
  fn operation(&self) -> &'static str {
    "filter-motions"
  }

  /// Create command for filtering omf motions.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to keep only selected motions of provided omf file")
      .arg(
        Arg::new("path")
          .help("Path to omf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting omf file")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("keep")
          .help("Exact motion names to keep")
          .short('k')
          .long("keep")
          .required(false)
          .num_args(1..)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("keep-prefix")
          .help("Keep motions whose name starts with provided prefix")
          .long("keep-prefix")
          .required(false)
          .num_args(1..)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("dry-run")
          .help("Validate the change and report the result without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  /// Keep only selected motions of provided omf file.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let destination: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output path to be provided");

    let names: Vec<String> = Self::collect_values(matches, "keep");
    let prefixes: Vec<String> = Self::collect_values(matches, "keep-prefix");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    Self::filter_file(
      &output,
      path,
      destination,
      &names,
      &prefixes,
      matches.get_flag("dry-run"),
    )?;

    Ok(())
  }
}

impl FilterMotionsCommand {
  fn collect_values(matches: &ArgMatches, id: &str) -> Vec<String> {
    matches
      .get_many::<String>(id)
      .map(|it| it.cloned().collect())
      .unwrap_or_default()
  }

  /// Filter motions of single omf file and write the result into destination.
  fn filter_file(
    output: &OutputOptions,
    path: &Path,
    destination: &Path,
    names: &[String],
    prefixes: &[String],
    is_dry_run: bool,
  ) -> XrfResult {
    if names.is_empty() && prefixes.is_empty() {
      return Err(XrfError::new_invalid_error(
        "Expected at least one of --keep or --keep-prefix to be provided",
      ));
    }

    let mut omf_file: Box<OmfFile> = Box::new(OmfFile::read_from_path::<XRayByteOrder, _>(&path)?);
    let original_count: usize = omf_file.motions.motions.len();

    let removed_count: usize = OmfMotionsProcessor::retain_motions(&mut omf_file, |name| {
      names.iter().any(|it| it == name) || prefixes.iter().any(|it| name.starts_with(it))
    })?;

    let retained_count: usize = original_count - removed_count;

    if retained_count == 0 {
      return Err(XrfError::new_invalid_error(format!(
        "Refused to filter {}, no motion matched the provided filters, {original_count} motions available",
        path.display()
      )));
    }

    xrf_output::info!(
      output,
      "Filtered omf motions {}, {retained_count}/{original_count} kept",
      path.display()
    );

    xrf_output::verbose!(output, "Kept motions: {}", omf_file.get_motion_names().join(","));

    if is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would receive {retained_count} motions",
        destination.display()
      );

      return Ok(());
    }

    omf_file.write_to_path::<XRayByteOrder, _>(&destination)?;

    xrf_output::info!(output, "Filtered omf file written into {}", destination.display());

    Ok(())
  }
}
