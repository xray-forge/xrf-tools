use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_translation::{ProjectBuildOptions, ProjectBuildResult, TranslationLanguage, build_dir, build_file};

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct BuildCommand;

impl GenericCommand for BuildCommand {
  fn operation(&self) -> &'static str {
    "build"
  }

  /// Create command for building of translation files.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to build translation files into gamedata")
      .arg(
        Arg::new("path")
          .help("Path to translation folder")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("output")
          .help("Path to output translation")
          .short('o')
          .long("output")
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
        Arg::new("sort")
          .help("Preserve source order instead of sorting dynamic translation files")
          .long("no-sort")
          .required(false)
          .action(ArgAction::SetFalse),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid path to be provided");

    let output_dir: &PathBuf = matches
      .get_one::<PathBuf>("output")
      .expect("Expected valid output folder path to be provided");

    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_sorted: bool = matches.get_flag("sort");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Building translation {}, language - {}, sorted - {}",
      path.display(),
      language,
      is_sorted
    );

    let options: ProjectBuildOptions = ProjectBuildOptions {
      is_sorted,
      output,
      path: path.clone(),
      output_dir: output_dir.clone(),
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
    };

    let result: ProjectBuildResult = if path.is_dir() {
      build_dir(path, &options)?
    } else {
      build_file(path, &options)?
    };

    xrf_output::info!(
      options.output,
      "Built translation files in {}",
      xrf_utils::format_duration(result.duration)
    );

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use clap::ArgMatches;

  use super::BuildCommand;
  use crate::core::generic_command::GenericCommand;

  fn parse_matches(extra: &[&str]) -> ArgMatches {
    let mut arguments = vec!["build", "--path", "translations", "--output", "output"];

    arguments.extend_from_slice(extra);

    BuildCommand.init().try_get_matches_from(arguments).unwrap()
  }

  #[test]
  fn translations_are_sorted_unless_source_order_is_requested() {
    assert!(parse_matches(&[]).get_flag("sort"));
    assert!(!parse_matches(&["--no-sort"]).get_flag("sort"));
  }
}
