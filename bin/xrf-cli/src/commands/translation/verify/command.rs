use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_report::Report;
use xrf_translation::{TranslationLanguage, TranslationVerifier, TranslationVerifyOptions, TranslationVerifyResult};
use xrf_utils::format_path;
use xrf_vfs::{XrayMountMode, XrayRoots};

use super::report::TranslationVerifyReport;
use crate::core::arguments::XrayRootsArguments;
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    XrayRootsArguments::declare(
      command.about(
        "Verify that translation sources build to tables the game loads as written, and are translated into every \
         language",
      ),
      "Root holding translation sources, or one source file. Repeat to layer roots, highest priority first",
      "Limit to one logical subtree, such as translations",
    )
    .arg(
      Arg::new("language")
        .help("Language to verify, or all of them")
        .short('l')
        .long("language")
        .required(false)
        .default_value("all")
        .value_parser(value_parser!(String)),
    )
    .arg(
      Arg::new("strict")
        .help("Also answer with a check failure when any translation is missing")
        .long("strict")
        .required(false)
        .action(ArgAction::SetTrue),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_strict: bool = matches.get_flag("strict");

    let output: OutputOptions = context.get_output().clone();

    let options: TranslationVerifyOptions = TranslationVerifyOptions {
      job: CliJob::for_output(&output),
      is_strict,
      output,
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
      // The command reports every missing translation by name, which is what its report is for.
      is_detailed: true,
    };

    let verify_result: XrfResult<TranslationVerifyResult> = match XrayRootsArguments::single_file_of(matches) {
      Some(file) => {
        xrf_output::info!(
          options.output,
          "Verifying translation file {}, language - {}",
          format_path(file),
          language
        );

        TranslationVerifier::verify_file(file, &options)
      }
      None => {
        let source: XrayMountMode = XrayRootsArguments::source_of(matches)?;
        let roots: XrayRoots = XrayRootsArguments::of(matches)?;

        xrf_output::info!(
          options.output,
          "Verifying translations in {} ({source}), language - {}",
          roots.describe(),
          language
        );

        TranslationVerifier::verify_roots(&roots, XrayRootsArguments::prefix_of(matches), &options)
      }
    };

    let result: TranslationVerifyResult = match verify_result {
      Ok(result) => result,
      // An unreadable source is an execution failure; only judged content is a check failure.
      Err(error @ XrfError::Io { .. }) => return Err(error.into()),
      Err(error) => {
        xrf_output::failure!(options.output, "Provided translations are invalid: {error}");
        context.set_result(|| TranslationVerifyReport::malformed(&error))?;

        return Err(CommandError::new_check_failed(1));
      }
    };

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings
    // that explain it.
    context.set_result(|| TranslationVerifyReport::new(&result))?;

    xrf_output::info!(
      options.output,
      "Verified translation files in {}, {} checked, {} missing, {} the game would refuse or mangle",
      xrf_utils::format_duration(result.duration),
      result.checked_translations_count,
      result.missing_translations_count,
      result.problems.len()
    );

    // Missing translations are reported and answered as success by default, as a project mid-translation always has
    // some and the game shows each one as its id; `--strict` is the mode that judges them. What the game would refuse
    // or mangle is judged either way.
    let report: Report = if options.is_strict {
      result.to_report()
    } else {
      result.to_engine_report()
    };

    ReportVerdict::print_findings(&options.output, &report);

    ReportVerdict::conclude(&options.output, &report, "Translation verification")
  }
}
