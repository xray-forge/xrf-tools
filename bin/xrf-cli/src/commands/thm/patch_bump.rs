use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{ThmBumpPatchReport, ThmBumpProcessor, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct PatchBumpCommand;

impl GenericCommand for PatchBumpCommand {
  fn operation(&self) -> &'static str {
    "patch-bump"
  }

  /// Create command for repointing the bump declaration of a thm file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to repoint the bump texture reference of provided thm file")
      .arg(
        Arg::new("path")
          .help("Path to thm file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting thm file, defaults to in place rewrite of the source file")
          .short('d')
          .long("dest")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("to")
          .help("Bump texture reference to write, engine style without extension, for example 'wpn\\wpn_pm\\wpn_pm_bump'")
          .long("to")
          .required(false),
      )
      .arg(
        Arg::new("off")
          .help("Declare no bump at all, clearing the mode and the name; use for a bump that does not exist and is not going to")
          .long("off")
          .action(ArgAction::SetTrue)
          .conflicts_with("to"),
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

  /// Repoint or disable the bump declaration of provided thm file.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let is_off: bool = matches.get_flag("off");
    let to: Option<&String> = matches.get_one::<String>("to");

    if to.is_none() && !is_off {
      return Err(
        XrfError::new_invalid_error("Expected either --to with a bump reference or --off to declare no bump").into(),
      );
    }

    let destination: &Path = matches
      .get_one::<PathBuf>("dest")
      .map_or(path.as_path(), |it| it.as_path());

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    Self::patch_file(
      &output,
      path,
      destination,
      to.map(String::as_str),
      matches.get_flag("dry-run"),
    )?;

    Ok(())
  }
}

impl PatchBumpCommand {
  /// Repoint the bump declaration of a single thm file, preserving all other chunks byte for byte.
  fn patch_file(
    output: &OutputOptions,
    path: &Path,
    destination: &Path,
    to: Option<&str>,
    is_dry_run: bool,
  ) -> XrfResult {
    let report: ThmBumpPatchReport = match to {
      Some(to) => ThmBumpProcessor::patch_bump_name_to_path::<XRayByteOrder>(path, destination, to, is_dry_run)?,
      None => ThmBumpProcessor::patch_bump_off_to_path::<XRayByteOrder>(path, destination, is_dry_run)?,
    };

    let outcome: String = match to {
      Some(to) => format!("bump '{}' -> '{}'", report.previous_name, to),
      None => format!("bump '{}' -> none", report.previous_name),
    };

    if report.is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} and {} would receive {} bytes instead of {}",
        outcome,
        destination.display(),
        report.patched_size,
        report.original_size
      );

      return Ok(());
    }

    xrf_output::info!(
      output,
      "Patched thm {}, written into {}",
      outcome,
      destination.display()
    );

    Ok(())
  }
}
