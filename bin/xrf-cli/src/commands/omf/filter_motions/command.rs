use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_omf::{OmfFile, OmfMotionsProcessor};
use xrf_output::OutputOptions;
use xrf_spawn::XRayByteOrder;
use xrf_utils::format_path;

use super::report::OmfFilterMotionsReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct FilterMotionsCommand;

impl GenericCommand for FilterMotionsCommand {
  fn operation(&self) -> &'static str {
    "filter-motions"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Keep only the selected motions of an OMF file")
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
          .num_args(1..)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("keep-prefix")
          .help("Keep motions whose name starts with provided prefix")
          .long("keep-prefix")
          .num_args(1..)
          .value_parser(value_parser!(String)),
      )
      // Either filter alone selects, and both together keep what either matches.
      .group(
        ArgGroup::new("filter")
          .args(["keep", "keep-prefix"])
          .required(true)
          .multiple(true),
      )
      .arg(
        Arg::new("dry-run")
          .help("Validate the change and report the result without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let destination: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output path to be provided");

    let names: Vec<String> = Self::collect_values(matches, "keep");
    let prefixes: Vec<String> = Self::collect_values(matches, "keep-prefix");

    Self::filter_file(
      context,
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
    context: &mut CommandContext,
    path: &Path,
    destination: &Path,
    names: &[String],
    prefixes: &[String],
    is_dry_run: bool,
  ) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    let mut omf_file: OmfFile = OmfFile::read_from_path::<XRayByteOrder, _>(&path)?;
    let original_count: usize = omf_file.motions.motions.len();

    let removed_count: usize = OmfMotionsProcessor::retain_motions(&mut omf_file, |name| {
      names.iter().any(|it| it == name) || prefixes.iter().any(|it| name.starts_with(it))
    })?;

    let retained_count: usize = original_count - removed_count;

    // Deposited before the refusal below and before the dry run returns, so a run that wrote
    // nothing still reports which motions the filters matched - which is what a caller tuning them
    // came for.
    context.set_result(|| OmfFilterMotionsReport::new(path, destination, &omf_file, original_count, is_dry_run))?;

    if retained_count == 0 {
      return Err(
        XrfError::new_invalid_error(format!(
          "Refused to filter {}, no motion matched the provided filters, {original_count} motions available",
          format_path(path)
        ))
        .into(),
      );
    }

    xrf_output::info!(
      output,
      "Filtered omf motions {}, {retained_count}/{original_count} kept",
      format_path(path)
    );

    xrf_output::verbose!(output, "Kept motions: {}", omf_file.get_motion_names().join(","));

    if is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would receive {retained_count} motions",
        format_path(destination)
      );

      return Ok(());
    }

    omf_file.write_to_path::<XRayByteOrder, _>(&destination)?;

    xrf_output::success!(output, "Filtered omf file written into {}", format_path(destination));

    Ok(())
  }
}
