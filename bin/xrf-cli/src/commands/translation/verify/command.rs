use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_translation::{ProjectVerifyOptions, ProjectVerifyResult, TranslationLanguage, verify_dir, verify_file};

use super::translation_verification_report::TranslationVerificationReportWriter;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  /// Create command for verifying of translation files.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to verify translation files integrity")
      .arg(
        Arg::new("path")
          .help("Path to translation folder")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("language")
          .help("Target language to translate")
          .short('l')
          .long("language")
          .required(false)
          .default_value("all")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("report")
          .help("Write the structured verification report as JSON")
          .long("report")
          .required(false)
          .value_name("PATH")
          .num_args(1)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("strict")
          .help("Fail with non 0 error code if translation are missing")
          .long("strict")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
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

  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid path to be provided");

    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_silent: bool = matches.get_flag("silent");
    let is_verbose: bool = matches.get_flag("verbose");
    let is_strict: bool = matches.get_flag("strict");
    let report_path: Option<PathBuf> = matches.get_one::<PathBuf>("report").cloned();

    let output: OutputOptions = TerminalOutput::from_options(is_silent, is_verbose);

    xrf_output::info!(
      output,
      "Verifying translation {}, language - {}",
      path.display(),
      language
    );

    let options: ProjectVerifyOptions = ProjectVerifyOptions {
      is_strict,
      output,
      path: path.clone(),
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
    };

    let verify_result: XrfResult<ProjectVerifyResult> = if path.is_dir() {
      verify_dir(path, &options)
    } else {
      verify_file(path, &options)
    };

    let result: ProjectVerifyResult = match verify_result {
      Ok(result) => result,
      // An unreadable source is an execution failure; only judged content is a check failure.
      Err(error @ XrfError::Io { .. }) => return Err(error.into()),
      Err(error) => {
        xrf_output::failure!(options.output, "Provided translations are invalid: {error}");

        return Err(CommandError::new_check_failed(1));
      }
    };

    if let Some(report_path) = report_path {
      TranslationVerificationReportWriter::new(&result).write(&report_path)?;
    }

    xrf_output::info!(
      options.output,
      "Verified translation files in {}, {} checked, {} missing",
      xrf_utils::format_duration(result.duration),
      result.checked_translations_count,
      result.missing_translations_count
    );

    if options.is_strict && result.missing_translations_count > 0 {
      return Err(CommandError::new_check_failed(
        result.missing_translations_count as usize,
      ));
    }

    Ok(())
  }
}
