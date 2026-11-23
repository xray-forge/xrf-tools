use std::sync::Arc;

use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::{format_duration, format_path};

use crate::project::events::{GamedataEvent, GamedataListener};
use crate::project::gamedata_verification_check_report::GamedataVerificationCheckReport;
use crate::project::gamedata_verification_status::GamedataVerificationStatus;
use crate::project::gamedata_verification_type::GamedataVerificationType;

/// What a person reads of a gamedata verification: each check's heading, what it found, and what it counted.
///
/// The checks themselves print nothing; this renders the reports they hand back, so a finding is worded once, by the
/// check that made it, and reads the same on the console and in the report.
pub struct GamedataRunPrinter;

impl GamedataRunPrinter {
  /// A listener printing every event to `output`, or none when nothing it prints would show. Findings are errors,
  /// which show at every verbosity, so in practice there always is one.
  pub fn new_listener(output: &OutputOptions) -> Option<GamedataListener> {
    if !output.is_visible(OutputChannel::Error) {
      return None;
    }

    let output: OutputOptions = output.clone();

    Some(Arc::new(move |event: &GamedataEvent<'_>| {
      Self::print_event(event, &output);
    }))
  }

  /// Says one event.
  pub fn print_event(event: &GamedataEvent<'_>, output: &OutputOptions) {
    match *event {
      GamedataEvent::Started { root, checks } => {
        xrf_output::info!(output, "Verifying gamedata project: {}", format_path(root));
        xrf_output::info!(
          output,
          "Verifying modules: \n  -{}",
          checks
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n  -")
        );
        xrf_output::info!(output, "");
      }
      GamedataEvent::CheckFinished { report } => Self::print_check(report, output),
    }
  }

  /// A check's heading, each finding as `subject: message`, and its summary; a check that could not finish says so
  /// in place of the summary.
  fn print_check(report: &GamedataVerificationCheckReport, output: &OutputOptions) {
    let label: &str = Self::get_label(report.get_verification_type());

    xrf_output::heading!(output, "Verify {label}:");

    for finding in report.get_findings() {
      match finding.subject() {
        Some(subject) => xrf_output::error!(output, "{subject}: {}", finding.message()),
        None => xrf_output::error!(output, "{}", finding.message()),
      }
    }

    match report.get_status() {
      GamedataVerificationStatus::Error => xrf_output::error!(output, "{}", report.get_summary()),
      GamedataVerificationStatus::Incomplete => xrf_output::warning!(output, "{}", report.get_summary()),
      GamedataVerificationStatus::Passed | GamedataVerificationStatus::Failed | GamedataVerificationStatus::Skipped => {
        match report.get_duration() {
          Some(duration) => xrf_output::info!(
            output,
            "Verified {label} in {}, {}",
            format_duration(duration),
            report.get_summary()
          ),
          None => xrf_output::info!(output, "Verified {label}, {}", report.get_summary()),
        }
      }
    }
  }

  /// What a check is called mid-sentence on the console.
  fn get_label(verification_type: GamedataVerificationType) -> &'static str {
    match verification_type {
      GamedataVerificationType::Collisions => "path collisions",
      GamedataVerificationType::Coverage => "declared source coverage",
      GamedataVerificationType::Animations => "animations",
      GamedataVerificationType::Dialogs => "dialogs",
      GamedataVerificationType::Environment => "environment",
      GamedataVerificationType::InfoPortions => "info portions",
      GamedataVerificationType::Levels => "levels",
      GamedataVerificationType::Ltx => "LTX files",
      GamedataVerificationType::Materials => "materials",
      GamedataVerificationType::Meshes => "meshes",
      GamedataVerificationType::Particles => "particles",
      GamedataVerificationType::ParticlesUsage => "particles usage",
      GamedataVerificationType::Scripts => "scripts",
      GamedataVerificationType::Shaders => "shaders",
      GamedataVerificationType::Sounds => "sounds",
      GamedataVerificationType::Spawns => "spawns",
      GamedataVerificationType::Textures => "textures",
      GamedataVerificationType::Weapons => "weapons",
    }
  }
}
