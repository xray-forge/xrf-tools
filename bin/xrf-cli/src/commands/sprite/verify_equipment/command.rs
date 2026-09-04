use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{EquipmentGridOverlap, VerifyEquipmentGridProcessor};

use super::report::SpriteEquipmentVerifyReport;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::ltx_dialect_selection::select_ltx_dialect;

#[derive(Default)]
pub struct VerifyEquipmentCommand;

impl GenericCommand for VerifyEquipmentCommand {
  fn operation(&self) -> &'static str {
    "verify-equipment"
  }

  /// Create command for verifying the inventory icon grid.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to check an equipment sprite's inventory icon grid rects for overlaps")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dltx")
          .help("Resolve configs with the Monolith/Anomaly DLTX patch dialect, applying mod_<base>_*.ltx files")
          .long("dltx")
          .required(false)
          .action(clap::ArgAction::SetTrue),
      )
  }

  /// Report inventory icon rects that overlap, exiting non zero when any are found.
  ///
  /// `pack-equipment` only warns when two sections write different art to the *same* slot.
  /// A rect widened into a neighbour's cells packs without complaint and silently overwrites it,
  /// so this is the check that has to run before widening or moving an icon.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("system-ltx")
      .expect("Expected valid path to system ltx to be provided");

    let output: OutputOptions = context.get_output().clone();

    // The system ltx is the judged content: an unparseable file fails the check, while an
    // unreadable one is an execution failure.
    let ltx: Ltx = match Ltx::read_from_file_with_dialect(path, select_ltx_dialect(matches.get_flag("dltx")).as_ref()) {
      Ok(ltx) => ltx,
      Err(error @ XrfError::Io { .. }) => return Err(error.into()),
      Err(error) => {
        xrf_output::failure!(output, "Provided system ltx is invalid: {error}");

        return Err(CommandError::new_check_failed(1));
      }
    };

    let overlaps: Vec<EquipmentGridOverlap> = VerifyEquipmentGridProcessor::find_overlaps(&ltx);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the overlaps
    // that explain it.
    context.set_result(|| SpriteEquipmentVerifyReport::new(path, &overlaps))?;

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
