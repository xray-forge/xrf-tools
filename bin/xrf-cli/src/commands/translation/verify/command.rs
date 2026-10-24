use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_translation::{TranslationLanguage, TranslationVerifier, TranslationVerifyOptions, TranslationVerifyResult};
use xrf_utils::format_path;
use xrf_vfs::{XrayMountMode, XrayRoots};

use super::report::TranslationVerifyReport;
use crate::core::arguments::XrayRootsArguments;
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    XrayRootsArguments::declare(
      command.about("Verify that translation sources are well formed and translated into every language"),
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
        .help("Answer with a check failure when any translation is missing")
        .long("strict")
        .required(false)
        .action(ArgAction::SetTrue),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<&PathBuf> = XrayRootsArguments::list_paths(matches);

    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_strict: bool = matches.get_flag("strict");

    let output: OutputOptions = context.get_output().clone();

    let options: TranslationVerifyOptions = TranslationVerifyOptions {
      job: CliJob::new_logging(),
      is_strict,
      output,
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
      // The command reports every missing translation by name, which is what its report is for.
      is_detailed: true,
    };

    // A single file keeps working through the path-taking reader: a VFS mounts a directory, never a
    // file, and one source with no mounted roots is the case that convenience exists for.
    let verify_result: XrfResult<TranslationVerifyResult> = if paths.len() == 1 && paths[0].is_file() {
      xrf_output::info!(
        options.output,
        "Verifying translation file {}, language - {}",
        format_path(&paths[0]),
        language
      );

      TranslationVerifier::verify_file(paths[0], &options)
    } else {
      let source: XrayMountMode = XrayRootsArguments::source_of(matches)?;
      let roots: XrayRoots = XrayRootsArguments::of(matches)?;

      xrf_output::info!(
        options.output,
        "Verifying translations in {} ({source}), language - {}",
        roots.describe(),
        language
      );

      TranslationVerifier::verify_roots(&roots, XrayRootsArguments::prefix_of(matches), &options)
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
