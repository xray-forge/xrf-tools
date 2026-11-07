use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_ogf::OgfFile;
use xrf_output::OutputOptions;
use xrf_rawdata::{ObjectGeometryCensus, ObjectVerification, ObjectVerifier};
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_utils::format_path;

use crate::commands::object::verify::report::ObjectVerifyReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

/// Compiles an SDK editable object in memory and checks the OGF against one it should stand in for, by what the
/// engine reads of it.
#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Compile a skeleton SDK .object in memory and compare it with the OGF it should stand in for")
      .arg(
        Arg::new("path")
          .help("Path to the .object to compile")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("ogf")
          .help("Path to the OGF the compile should stand in for, such as the one the object was exported to")
          .long("ogf")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = matches.get_one::<PathBuf>("path").expect("the path is required");
    let reference: &PathBuf = matches.get_one::<PathBuf>("ogf").expect("the OGF is required");
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Verifying {} against {}",
      format_path(source),
      format_path(reference)
    );

    let started: Instant = Instant::now();
    let verification: ObjectVerification = Self::verify(source, reference)?;
    let report: Report = Self::report(&verification, started.elapsed());
    let census: &ObjectGeometryCensus = &verification.census;

    xrf_output::info!(
      output,
      "Children {} against {}; vertices matched {} of {} ({} more but for their normal); reference faces drawn {} of \
       {} ({} with two corners at one position)",
      census.children,
      census.reference_children,
      census.vertices_matched,
      census.vertices,
      census.vertices_normal_only,
      census.faces_matched,
      census.reference_faces,
      census.reference_flat_faces
    );
    ReportVerdict::print_findings(&output, &report);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings that explain it.
    context.set_result(|| ObjectVerifyReport::new(census, &report))?;

    ReportVerdict::conclude(&output, &report, "Verification of the object")
  }
}

impl VerifyCommand {
  fn verify(source: &Path, reference: &Path) -> XrfResult<ObjectVerification> {
    let object: EditableObjectFile = EditableObjectFile::read_from_path::<XRayByteOrder, _>(&source)?;
    let reference: OgfFile = OgfFile::read_from_path::<XRayByteOrder, _>(&reference)?;

    ObjectVerifier::verify(&object, &reference)
  }

  /// One check per area the engine reads, failed by any difference in it.
  pub(crate) fn report(verification: &ObjectVerification, duration: Duration) -> Report {
    let check = |id: &'static str, differences: &[String]| {
      CheckReport::new(
        CheckId::of_static(id),
        Status::from_is_valid(differences.is_empty()),
        Some(duration),
        differences
          .iter()
          .map(|difference| Finding::new(RuleId::of_static(id), None, difference.clone()))
          .collect(),
      )
    };

    Report::new(vec![
      check("object.model", &verification.model),
      check("object.skeleton", &verification.skeleton),
      check("object.motions", &verification.motions),
      check("object.geometry", &verification.geometry),
    ])
  }
}
