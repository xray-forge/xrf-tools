use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_translation::{ProjectBuildOptions, ProjectBuildResult, TranslationLanguage, build_file, build_roots};
use xrf_utils::format_path;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

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
    let paths: Vec<&PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected at least one path to be provided")
      .collect();

    let output_dir: &PathBuf = matches
      .get_one::<PathBuf>("output")
      .expect("Expected valid output folder path to be provided");

    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_sorted: bool = matches.get_flag("sort");
    let output: OutputOptions = context.get_output().clone();

    let options: ProjectBuildOptions = ProjectBuildOptions {
      is_sorted,
      output,
      output_dir: output_dir.clone(),
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
    };

    // A single source keeps working through the path-taking reader: a VFS mounts a directory, never a
    // file, and one source with no mounted roots is the case that convenience exists for.
    let result: ProjectBuildResult = if paths.len() == 1 && paths[0].is_file() {
      xrf_output::info!(
        options.output,
        "Building translation {}, language - {}, sorted - {}",
        format_path(&paths[0]),
        language,
        is_sorted
      );

      build_file(paths[0], &options)?
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
        "Building translations in {} ({:?}), language - {}, sorted - {}",
        roots.describe(),
        source,
        language,
        is_sorted
      );

      build_roots(
        &roots,
        matches.get_one::<String>("prefix").map(String::as_str),
        &options,
      )?
    };

    xrf_output::info!(
      options.output,
      "Built translation files in {}",
      xrf_utils::format_duration(result.duration)
    );

    context.set_result(|| &result)?;

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
