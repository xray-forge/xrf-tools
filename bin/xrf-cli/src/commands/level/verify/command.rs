use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_report::Status;
use xrf_utils::format_path;

use crate::commands::level::verify::level_verification_result::LevelVerificationResult;
use crate::commands::level::verify::level_verifier::LevelVerifier;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to verify a compiled level's geometry can be drawn the way the renderer addresses it")
      .arg(
        Arg::new("path")
          .help("Path to a compiled level directory, the one holding `level` and `level.geom`")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Read every drawable visual's range and report what the renderer could not draw.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Verifying compiled level {}", format_path(path));

    let result: LevelVerificationResult = LevelVerifier::new(path).run();

    Self::print_census(&output, &result);
    Self::print_findings(&output, &result);

    // Deposited before the verdict becomes an outcome, so a failing check still reports what explains it.
    context.set_result(|| &result.report)?;

    let status: Status = result.report.status();

    match status {
      Status::Passed => {
        xrf_output::success!(output, "Verification passed, status: {}", status);

        Ok(())
      }
      Status::Failed => {
        let findings: usize = result
          .report
          .checks()
          .iter()
          .map(|check| check.findings().len())
          .sum::<usize>()
          .max(1);

        Err(CommandError::new_check_failed(findings))
      }
      Status::Error | Status::Incomplete | Status::Skipped => {
        Err(XrfError::new_verify_error(format!("Verification of the level did not complete, status: {status}")).into())
      }
    }
  }
}

impl VerifyCommand {
  fn print_census(output: &OutputOptions, result: &LevelVerificationResult) {
    xrf_output::info!(
      output,
      "Read {} visuals in {}, {} drawable and {} with a fast path",
      result.census.visuals,
      xrf_utils::format_duration(result.duration),
      result.census.drawable_visuals,
      result.census.fastpath_visuals
    );
    xrf_output::info!(
      output,
      "Addressed {} vertices and {} indices against a {} entry shader table",
      result.census.vertices,
      result.census.indices,
      result.census.shader_entries
    );

    xrf_output::info!(
      output,
      "{} sectors joined by {} portals, named {} times between them",
      result.census.sectors,
      result.census.portals,
      result.census.sector_portal_references
    );

    xrf_output::info!(
      output,
      "{} static lights{}",
      result.census.lights,
      if result.census.has_sun {
        ", one of them the sun"
      } else {
        ", none of them a sun"
      }
    );

    if let Some((min, max)) = &result.census.bounds {
      xrf_output::info!(
        output,
        "Decoded geometry spans ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1}), widest texture coordinate {:.1} tiles",
        min.x,
        min.y,
        min.z,
        max.x,
        max.y,
        max.z,
        result.census.widest_coordinate
      );
    }

    if result.census.stray_normals > 0 {
      xrf_output::warning!(
        output,
        "{} decoded normals are not unit length, which a misread normal element would explain",
        result.census.stray_normals
      );
    }

    for (layout, count) in &result.census.layouts {
      xrf_output::verbose!(output, "Layout {layout}: {count} visuals");
    }

    for (model_type, count) in &result.census.model_types {
      xrf_output::verbose!(output, "{model_type}: {count} visuals");
    }
  }

  fn print_findings(output: &OutputOptions, result: &LevelVerificationResult) {
    for check in result.report.checks() {
      for finding in check.findings() {
        xrf_output::warning!(output, "[{}] {}", finding.rule_id(), finding.message());
      }
    }
  }
}
