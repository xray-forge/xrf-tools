use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_report::Status;

use crate::commands::ogf::verify_ogf::ogf_verifier::{OgfVerificationCensus, OgfVerificationResult, OgfVerifier};
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct VerifyOgfCommand;

impl GenericCommand for VerifyOgfCommand {
  fn name(&self) -> &'static str {
    "verify-ogf"
  }

  fn init(&self) -> Command {
    Command::new(self.name())
      .about("Command to verify ogf visuals can be packed for rendering")
      .arg(
        Arg::new("path")
          .help("Path to an ogf file or a directory to sweep")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("report")
          .help("Path to write the verification report as json")
          .short('r')
          .long("report")
          .value_parser(value_parser!(PathBuf)),
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

  /// Pack every visual under the provided path and report what could not be drawn.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");
    let report_path: Option<&PathBuf> = matches.get_one::<_>("report");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    xrf_output::info!(output, "Verifying ogf visuals in {}", path.display());

    let result: OgfVerificationResult = OgfVerifier::new(path).run();

    Self::print_census(&output, &result);
    Self::print_findings(&output, &result);

    if let Some(report_path) = report_path {
      std::fs::write(
        report_path,
        format!("{}\n", serde_json::to_string_pretty(&result.report)?),
      )?;

      xrf_output::info!(output, "Wrote report to {}", report_path.display());
    }

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
      // The sweep did not judge the content to the end, so this is an execution failure rather
      // than a check verdict.
      Status::Error | Status::Incomplete | Status::Skipped => Err(
        XrfError::new_verify_error(format!(
          "Verification of ogf visuals did not complete, status: {status}"
        ))
        .into(),
      ),
    }
  }
}

impl VerifyOgfCommand {
  fn print_census(output: &OutputOptions, result: &OgfVerificationResult) {
    let census: &OgfVerificationCensus = &result.census;

    xrf_output::info!(
      output,
      "Swept {} visuals in {}",
      census.files,
      xrf_utils::format_duration(result.duration)
    );
    xrf_output::info!(
      output,
      "Submeshes: {} total, {} packed, {} unsupported, {} malformed",
      census.submeshes,
      census.packed_submeshes,
      census.unsupported_submeshes,
      census.malformed_submeshes
    );
    xrf_output::info!(
      output,
      "Progressive submeshes: {}, of which {} draw only part of their index buffer",
      census.progressive_submeshes,
      census.progressive_submeshes_drawing_part_of_the_buffer
    );
    xrf_output::info!(
      output,
      "Skinned submeshes: {}, vertices whose weights do not sum to one: {}",
      census.skinned_submeshes,
      census.vertices_with_stray_skin_weights
    );
    xrf_output::info!(
      output,
      "Unreadable files: {}, files with nothing drawable: {}, bounds disagreements: {}",
      census.unreadable_files,
      census.files_without_geometry,
      census.bounds_disagreements
    );

    xrf_output::info!(
      output,
      "Texture references: {} total, {} resolved, {} missing, {} unreadable, {} visuals with no root",
      census.texture_references,
      census.resolved_texture_references,
      census.missing_texture_references,
      census.unreadable_textures,
      census.visuals_without_root
    );
    xrf_output::info!(
      output,
      "Distinct textures: {}, of which {} carry no mip chain",
      census.distinct_textures,
      census.textures_without_mipmaps
    );

    Self::print_distribution(output, "Header versions", &census.versions);
    Self::print_distribution(output, "Root model types", &census.root_model_types);
    Self::print_distribution(output, "Submesh model types", &census.submesh_model_types);
    Self::print_distribution(output, "Vertex formats", &census.vertex_formats);
    Self::print_distribution(output, "Texture formats", &census.texture_formats);
    Self::print_distribution(output, "Texture sizes", &census.texture_sizes);
  }

  fn print_distribution<K: std::fmt::Display>(output: &OutputOptions, label: &str, counts: &BTreeMap<K, usize>) {
    let rendered: Vec<String> = counts.iter().map(|(key, count)| format!("{key}: {count}")).collect();

    xrf_output::info!(output, "{}: {}", label, rendered.join(", "));
  }

  /// List findings under verbose output only: a sweep of thousands of files can produce more of them
  /// than a terminal is useful for, and the json report is the artifact meant for comparison.
  fn print_findings(output: &OutputOptions, result: &OgfVerificationResult) {
    for check in result.report.checks() {
      if check.findings().is_empty() {
        continue;
      }

      xrf_output::info!(
        output,
        "Check '{}' is {} with {} findings",
        check.id(),
        check.status(),
        check.findings().len()
      );

      for finding in check.findings() {
        xrf_output::verbose!(
          output,
          "  [{}] {}: {}",
          finding.rule_id(),
          finding.subject().unwrap_or("-"),
          finding.message()
        );
      }
    }
  }
}
