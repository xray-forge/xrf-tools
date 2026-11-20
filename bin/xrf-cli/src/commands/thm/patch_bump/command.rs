use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_thm::{ThmBumpChunk, ThmBumpMode, ThmFile, ThmTextureEdit};
use xrf_utils::{StagedFile, format_path};

use super::report::ThmPatchBumpReport;
use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct PatchBumpCommand;

impl GenericCommand for PatchBumpCommand {
  fn operation(&self) -> &'static str {
    "patch-bump"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to thm file")
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Repoint the bump texture reference of a THM file")
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
          .long("to"),
      )
      .arg(
        Arg::new("off")
          .help("Declare no bump at all, clearing the mode and the name; use for a bump that does not exist and is not going to")
          .long("off")
          .action(ArgAction::SetTrue),
      )
      .group(ArgGroup::new("bump").args(["to", "off"]).required(true))
      .arg(
        Arg::new("dry-run")
          .help("Validate the change and report the result without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = InputArgument::of(matches);
    let to: Option<&str> = matches.get_one::<String>("to").map(String::as_str);
    let destination: &Path = matches
      .get_one::<PathBuf>("dest")
      .map_or(path.as_path(), |it| it.as_path());
    let is_dry_run: bool = matches.get_flag("dry-run");
    let output: OutputOptions = context.get_output().clone();
    let original: Vec<u8> = std::fs::read(path).map_err(|error| {
      XrfError::new_io_error(
        format!("THM file was not read: {}, error: {error}", format_path(path)),
        error.kind(),
      )
    })?;
    let previous: ThmBumpChunk = Self::read_bump(path, &original)?;
    let patched: Vec<u8> = Self::edit_of(to).rewrite::<XRayByteOrder>(&original)?.ok_or_else(|| {
      XrfError::new_verify_error(format!(
        "Refused to patch {}, which would not be written back byte for byte, so its other chunks could not be kept",
        format_path(path)
      ))
    })?;

    if !is_dry_run {
      StagedFile::write_creating_directories(destination, &patched)?;
    }

    let report: ThmPatchBumpReport = ThmPatchBumpReport {
      original_size: original.len(),
      patched_size: patched.len(),
      previous_name: previous.name,
      previous_mode: previous.mode,
      is_dry_run,
    };
    let outcome: String = match to {
      Some(to) => format!("bump '{}' -> '{}'", report.previous_name, to),
      None => format!("bump '{}' -> none", report.previous_name),
    };

    context.set_result(|| &report)?;

    if is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} and {} would receive {} bytes instead of {}",
        outcome,
        format_path(destination),
        report.patched_size,
        report.original_size
      );
    } else {
      xrf_output::success!(
        output,
        "Patched thm {}, written into {}",
        outcome,
        format_path(destination)
      );
    }

    Ok(())
  }
}

impl PatchBumpCommand {
  /// The bump declaration the descriptor `original` holds, which a descriptor without one cannot be patched to.
  fn read_bump(path: &Path, original: &[u8]) -> XrfResult<ThmBumpChunk> {
    ThmFile::read_from_bytes::<XRayByteOrder>(original.to_vec())?
      .bump
      .ok_or_else(|| XrfError::new_not_found_error(format!("THM file declares no bump chunk: {}", format_path(path))))
  }

  /// The edit `--to` or `--off` asks for.
  ///
  /// `--to` repoints the name and keeps the mode, so a declaration the SDK reads as off stays off; `--off` clears the
  /// name and sets the mode to none, the form `STextureParams` writes for a texture without a bump.
  fn edit_of(to: Option<&str>) -> ThmTextureEdit {
    match to {
      Some(to) => ThmTextureEdit {
        bump_name: Some(to.to_owned()),
        ..ThmTextureEdit::default()
      },
      None => ThmTextureEdit {
        bump_mode: Some(ThmBumpMode::None),
        bump_name: Some(String::new()),
        ..ThmTextureEdit::default()
      },
    }
  }
}
