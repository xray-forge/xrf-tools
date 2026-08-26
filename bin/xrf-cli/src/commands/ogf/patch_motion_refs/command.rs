use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{OgfFile, OgfMotionRefsProcessor, OgfRefsPatchReport, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_output::OutputOptions;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct PatchMotionRefsCommand;

impl GenericCommand for PatchMotionRefsCommand {
  fn operation(&self) -> &'static str {
    "patch-motion-refs"
  }

  /// Create command for rewriting ogf motion refs.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to rewrite motion refs of provided ogf file")
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
          .short('r')
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

  /// Rewrite motion refs of provided ogf file.
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
    let existing: Vec<String> = OgfFile::read_motion_refs_from_path::<XRayByteOrder, _>(&path)?;

    xrf_output::info!(
      output,
      "Patch ogf motion refs {}, {:?} -> {:?}",
      path.display(),
      existing,
      motion_refs
    );

    let report: OgfRefsPatchReport =
      OgfMotionRefsProcessor::patch_motion_refs_to_path::<XRayByteOrder>(path, destination, motion_refs, is_dry_run)?;

    if report.is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would receive {} bytes instead of {}",
        destination.display(),
        report.patched_size,
        report.original_size
      );

      return Ok(report);
    }

    xrf_output::info!(output, "Ogf motion refs written into {}", destination.display());

    Ok(report)
  }
}
