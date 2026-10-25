use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_texture::{EquipmentGridOverlap, EquipmentGridVerifier};
use xrf_utils::to_portable_path_string;

use crate::commands::sprite::verify_equipment::SpriteVerifyEquipmentReport;
use crate::core::arguments::LtxDialectArgument;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

#[derive(Default)]
pub struct VerifyEquipmentCommand;

impl VerifyEquipmentCommand {
  /// The one check this command runs: no two sections' icon rects cover a common cell.
  const CHECK: &'static str = "equipment.overlaps";
  /// Two sections whose rects cover a common cell.
  const RULE_OVERLAP: &'static str = "equipment.overlap";
  /// A system ltx that does not parse, so nothing in it was checked.
  const RULE_LTX: &'static str = "equipment.ltx";

  /// What two overlapping rects are found as: the pair is the subject, the cells they share the message.
  fn to_finding(overlap: &EquipmentGridOverlap) -> Finding {
    Finding::new(
      RuleId::of_static(Self::RULE_OVERLAP),
      Some(format!("{} / {}", overlap.first, overlap.second)),
      format!(
        "{} cell(s) shared, from {}:{}",
        overlap.overlapping_cells, overlap.cell.0, overlap.cell.1
      ),
    )
  }
}

impl GenericCommand for VerifyEquipmentCommand {
  fn operation(&self) -> &'static str {
    "verify-equipment"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Check an equipment sprite's inventory icon grid rects for overlaps")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      );

    LtxDialectArgument::declare(command)
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
    let started_at: Instant = Instant::now();

    // The system ltx is the judged content: an unparseable file fails the check, while an
    // unreadable one is an execution failure.
    let findings: Vec<Finding> = match Ltx::read_from_file_with_dialect(path, LtxDialectArgument::of(matches).as_ref())
    {
      Ok(ltx) => EquipmentGridVerifier::find_overlaps(&ltx)
        .iter()
        .map(Self::to_finding)
        .collect(),
      Err(error @ XrfError::Io { .. }) => return Err(error.into()),
      Err(error) => vec![Finding::new(
        RuleId::of_static(Self::RULE_LTX),
        Some(to_portable_path_string(path)),
        error.to_string(),
      )],
    };
    let report: Report = Report::new(vec![CheckReport::new(
      CheckId::of_static(Self::CHECK),
      Status::from_is_valid(findings.is_empty()),
      Some(started_at.elapsed()),
      findings,
    )]);

    ReportVerdict::print_findings(&output, &report);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the overlaps that explain it.
    context.set_result(|| SpriteVerifyEquipmentReport::new(path, report.clone()))?;

    ReportVerdict::conclude(&output, &report, "Inventory icon grid")
  }
}
