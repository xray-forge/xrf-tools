use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfResult;
use xrf_ogf::{OgfMotionRefsProcessor, OgfRefsPatchReport};
use xrf_output::OutputOptions;
use xrf_spawn::XRayByteOrder;
use xrf_utils::format_path;

use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct PatchMotionRefsCommand;

impl GenericCommand for PatchMotionRefsCommand {
  fn operation(&self) -> &'static str {
    "patch-motion-refs"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Rewrite the motion refs of an OGF visual")
      .arg(
        Arg::new("path")
          .help("Path to ogf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting ogf file, defaults to in place rewrite of the source file")
          .short('d')
          .long("dest")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("refs")
          .help("Motion refs to store in the ogf file")
          .long("refs")
          .required(true)
          .num_args(1..)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("dry-run")
          .help("Validate the rewrite and report the result without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let motion_refs: Vec<String> = matches
      .get_many::<String>("refs")
      .expect("Expected valid motion refs to be provided")
      .cloned()
      .collect();

    let destination: &Path = matches
      .get_one::<PathBuf>("dest")
      .map_or(path.as_path(), |it| it.as_path());

    let output: OutputOptions = context.get_output().clone();

    let report: OgfRefsPatchReport =
      Self::report_patch_file(&output, path, destination, &motion_refs, matches.get_flag("dry-run"))?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl PatchMotionRefsCommand {
  /// Rewrite motion refs of single ogf file, preserving all other chunks byte for byte.
  fn report_patch_file(
    output: &OutputOptions,
    path: &Path,
    destination: &Path,
    motion_refs: &[String],
    is_dry_run: bool,
  ) -> XrfResult<OgfRefsPatchReport> {
    let report: OgfRefsPatchReport =
      OgfMotionRefsProcessor::patch_motion_refs_to_path::<XRayByteOrder>(path, destination, motion_refs, is_dry_run)?;

    xrf_output::info!(
      output,
      "Patch ogf motion refs {}, [{}] -> [{}]",
      format_path(path),
      report.previous_refs.join(", "),
      motion_refs.join(", ")
    );

    if report.is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would receive {} bytes instead of {}",
        format_path(destination),
        report.patched_size,
        report.original_size
      );

      return Ok(report);
    }

    xrf_output::success!(output, "Ogf motion refs written into {}", format_path(destination));

    Ok(report)
  }
}
