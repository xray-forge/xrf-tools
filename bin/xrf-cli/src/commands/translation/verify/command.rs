use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_translation::{ProjectVerifyOptions, ProjectVerifyResult, TranslationLanguage, verify_file, verify_roots};
use xrf_utils::format_path;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use super::translation_verification_report::TranslationVerificationReportPayload;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};

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
          .help("Root holding translation sources, or one source file. Repeat to layer roots, highest priority first")
          .short('p')
          .long("path")
          .required(true)
          // Both spellings layer: repeat the flag, or list several values after one of them.
          .action(ArgAction::Append)
          .num_args(1..)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("source")
          .help(
            "How to read the path: auto treats it as an installation only when it declares one, directory ignores any declaration, volumes mounts every archive volume beneath it, installation requires one, containing-installation searches parent directories for one",
          )
          .long("source")
          .default_value("containing-installation")
          .value_parser(["auto", "directory", "volumes", "installation", "containing-installation"]),
      )
      .arg(
        Arg::new("prefix")
          .help("Limit to one logical subtree, such as translations")
          .long("prefix")
          .value_parser(value_parser!(String)),
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
        Arg::new("strict")
          .help("Fail with non 0 error code if translation are missing")
          .long("strict")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<&PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected at least one path to be provided")
      .collect();

    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_strict: bool = matches.get_flag("strict");

    let output: OutputOptions = context.get_output().clone();

    let options: ProjectVerifyOptions = ProjectVerifyOptions {
      is_strict,
      output,
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
      // The command reports every missing translation by name, which is what its report is for.
      is_detailed: true,
    };

    // A single file keeps working through the path-taking reader: a VFS mounts a directory, never a
    // file, and one source with no mounted roots is the case that convenience exists for.
    let verify_result: XrfResult<ProjectVerifyResult> = if paths.len() == 1 && paths[0].is_file() {
      xrf_output::info!(
        options.output,
        "Verifying translation file {}, language - {}",
        format_path(&paths[0]),
        language
      );

      verify_file(paths[0], &options)
    } else {
      let source: XrayMountMode = XrayMountMode::try_from(
        matches
          .get_one::<String>("source")
          .expect("Expected source mode to default")
          .as_str(),
      )?;

      // One vocabulary for naming roots, so repeating `--path` layers a tree in front of an
      // installation exactly as the desktop app does it.
      let roots: XrayRoots = XrayRoots::new(
        paths
          .iter()
          .map(|path| XrayRoot::new(path.display().to_string(), source)),
      );

      xrf_output::info!(
        options.output,
        "Verifying translations in {} ({:?}), language - {}",
        roots.describe(),
        source,
        language
      );

      verify_roots(
        &roots,
        matches.get_one::<String>("prefix").map(String::as_str),
        &options,
      )
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

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings
    // that explain it.
    context.set_result(|| TranslationVerificationReportPayload::new(&result).build())?;

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
