use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_gamedata::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationResult,
  GamedataVerificationStatus, GamedataVerificationType,
};
use xrf_output::OutputOptions;

use super::verification_report::GamedataVerificationReportPayload;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// How many of the most-read paths `--trace-reads` names individually.
///
/// A sweep touches tens of thousands, and the redundancy worth chasing has always been concentrated: four animation
/// banks accounted for 47GB of one Anomaly run. The summary reports the untruncated path count beside the list.
const HOTTEST_READ_PATHS_REPORTED: usize = 25;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  /// Create command to verify gamedata.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to verify gamedata")
      .arg(
        Arg::new("root")
          .help("Path to assembled gamedata root")
          .required(true)
          .value_name("ROOT")
          .num_args(1)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("ignore")
          .help("Ignored assets in the gamedata root")
          .short('i')
          .long("ignore")
          .required(false)
          .value_delimiter(',')
          .num_args(1..=10)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("checks")
          .help("List of checks to perform")
          .long("checks")
          .value_delimiter(',')
          .num_args(1..)
          .value_parser(value_parser!(GamedataVerificationType)),
      )
      .arg(
        Arg::new("strict")
          .help("Fully validate expensive asset payloads")
          .long("strict")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("trace-reads")
          .help("Account for every asset read, reporting redundancy against unique paths")
          .long("trace-reads")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Unpack xray engine database archive.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let root: PathBuf = matches
      .get_one::<PathBuf>("root")
      .expect("Expected a valid gamedata root to be provided")
      .clone();

    let ignored: Vec<String> = matches
      .get_many::<String>("ignore")
      .map(|it| it.cloned().collect::<Vec<String>>())
      .unwrap_or_else(|| {
        vec![
          String::from(".git"),
          String::from(".idea"),
          String::from("particles_unpacked"),
          String::from("textures_unpacked"),
          String::from(".gitignore"),
          String::from(".gitattributes"),
          String::from("README.md"),
          String::from("LICENSE"),
        ]
      });

    let checks: Vec<GamedataVerificationType> = matches
      .get_many::<GamedataVerificationType>("checks")
      .map(|it| it.cloned().collect::<Vec<_>>())
      .unwrap_or_else(GamedataVerificationType::get_all);

    let output: OutputOptions = context.get_output().clone();
    let is_strict: bool = matches.get_flag("strict");

    let open_options: GamedataProjectReadOptions = GamedataProjectReadOptions {
      root: root.clone(),
      ignored,
      output: output.clone(),
      is_strict,
      is_tracing_reads: matches.get_flag("trace-reads"),
    };

    let verify_options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      output,
      is_strict,
      checks,
    };

    xrf_output::heading!(open_options.output, "Opening gamedata project");
    xrf_output::info!(
      open_options.output,
      "Root: {}, ignored: [{}]",
      open_options.root.display(),
      open_options.ignored.join(", "),
    );

    let project: Box<GamedataProject> = Box::new(GamedataProject::open(&open_options)?);
    let verify_result: GamedataVerificationResult = project.verify(&verify_options)?;
    let status: GamedataVerificationStatus = verify_result.get_status();

    // Deposited before the verdict is turned into an outcome, so a failing check still reports the findings.
    context.set_result(|| {
      GamedataVerificationReportPayload::new(
        &root,
        &verify_result,
        project.get_cache_stats(),
        project.get_read_trace_summary(HOTTEST_READ_PATHS_REPORTED),
      )
      .build()
    })?;

    match status {
      GamedataVerificationStatus::Passed => {
        xrf_output::info!(verify_options.output, "");
        xrf_output::success!(verify_options.output, "Project gamedata is valid");
        xrf_output::info!(
          verify_options.output,
          "Gamedata project verified in {}",
          xrf_utils::format_duration(verify_result.get_duration())
        );

        Ok(())
      }
      GamedataVerificationStatus::Failed
      | GamedataVerificationStatus::Error
      | GamedataVerificationStatus::Incomplete
      | GamedataVerificationStatus::Skipped => {
        xrf_output::error!(verify_options.output, "");

        let status_message = match status {
          GamedataVerificationStatus::Failed => "Project gamedata is invalid",
          GamedataVerificationStatus::Error => "Project gamedata verification has errors",
          GamedataVerificationStatus::Incomplete => "Project gamedata verification is incomplete",
          GamedataVerificationStatus::Skipped => "Project gamedata verification was skipped",
          GamedataVerificationStatus::Passed => unreachable!(),
        };

        match status {
          GamedataVerificationStatus::Failed | GamedataVerificationStatus::Error => {
            verify_options.output.failure(status_message);
          }
          GamedataVerificationStatus::Incomplete | GamedataVerificationStatus::Skipped => {
            verify_options.output.warning(status_message);
          }
          GamedataVerificationStatus::Passed => unreachable!(),
        }

        for message in verify_result.get_failure_messages() {
          xrf_output::error!(verify_options.output, "- {message}");
        }

        for report in verify_result.get_failure_reports() {
          for finding in report.get_findings() {
            match finding.subject() {
              Some(subject) => xrf_output::error!(
                verify_options.output,
                "  - [{}] {}: {}",
                report.get_verification_type(),
                subject,
                finding.message()
              ),
              None => xrf_output::error!(
                verify_options.output,
                "  - [{}] {}",
                report.get_verification_type(),
                finding.message()
              ),
            }
          }
        }

        xrf_output::error!(
          verify_options.output,
          "Gamedata project checked in {}",
          xrf_utils::format_duration(verify_result.get_duration())
        );

        if status == GamedataVerificationStatus::Failed {
          let findings: usize = verify_result
            .get_failure_reports()
            .map(|report| report.get_findings().len())
            .sum::<usize>()
            .max(1);

          Err(CommandError::new_check_failed(findings))
        } else {
          // Error, incomplete, and skipped runs did not judge the content to the end, so they are
          // execution failures rather than check verdicts.
          Err(XrfError::new_verify_error(status_message).into())
        }
      }
    }
  }
}
