use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{EquipmentGridOverlap, VerifyEquipmentGridProcessor};

use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct VerifyEquipmentIconsCommand;

impl GenericCommand for VerifyEquipmentIconsCommand {
  fn operation(&self) -> &'static str {
    "verify-equipment-icons"
  }

  /// Create command for verifying the inventory icon grid.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to check inventory icon grid rects for overlaps")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Turn off logging")
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

  /// Report inventory icon rects that overlap, exiting non zero when any are found.
  ///
  /// `pack-equipment-icons` only warns when two sections write different art to the *same* slot.
  /// A rect widened into a neighbour's cells packs without complaint and silently overwrites it,
  /// so this is the check that has to run before widening or moving an icon.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("system-ltx")
      .expect("Expected valid path to system ltx to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    // The system ltx is the judged content: an unparseable file fails the check, while an
    // unreadable one is an execution failure.
    let ltx: Ltx = match Ltx::read_from_file_full(path) {
      Ok(ltx) => ltx,
      Err(error @ XrfError::Io { .. }) => return Err(error.into()),
      Err(error) => {
        xrf_output::failure!(output, "Provided system ltx is invalid: {error}");

        return Err(CommandError::new_check_failed(1));
      }
    };

    let overlaps: Vec<EquipmentGridOverlap> = VerifyEquipmentGridProcessor::find_overlaps(&ltx);

    if overlaps.is_empty() {
      xrf_output::info!(output, "Inventory icon grid is clean, no overlapping rects");

      return Ok(());
    }

    for overlap in &overlaps {
      xrf_output::error!(
        output,
        "Overlapping icon rects at {}:{}, {} cell(s) shared by '{}' and '{}'",
        overlap.cell.0,
        overlap.cell.1,
        overlap.overlapping_cells,
        overlap.first,
        overlap.second
      );
    }

    xrf_output::error!(output, "Found {} overlapping icon rect pair(s)", overlaps.len());

    Err(CommandError::new_check_failed(overlaps.len()))
  }
}
