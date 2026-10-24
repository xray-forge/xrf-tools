use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_translation::{TranslationBuildOptions, TranslationBuildResult, TranslationBuilder, TranslationLanguage};
use xrf_utils::format_path;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::core::arguments::{LinkPolicyArgument, XrayRootsArguments};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct BuildCommand;

impl GenericCommand for BuildCommand {
  fn operation(&self) -> &'static str {
    "build"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = XrayRootsArguments::declare(
      command.about("Build translation files into gamedata"),
      "Root holding translation sources, or one source file. Repeat to layer roots, highest priority first",
      "Limit to one logical subtree, such as translations",
    )
    .arg(
      Arg::new("dest")
        .help("Directory the built string tables are written to")
        .short('d')
        .long("dest")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
      Arg::new("language")
        .help("Language to build, or all of them")
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
    );

    LinkPolicyArgument::declare(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<&PathBuf> = XrayRootsArguments::list_paths(matches);

    let output_dir: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output folder path to be provided");

    let language: &String = matches
      .get_one::<String>("language")
      .expect("Expected valid language for translation");

    let is_sorted: bool = matches.get_flag("sort");
    let output: OutputOptions = context.get_output().clone();

    let options: TranslationBuildOptions = TranslationBuildOptions {
      job: CliJob::new_logging(),
      is_sorted,
      output,
      output_dir: output_dir.clone(),
      language: TranslationLanguage::from_str(language).map_err(XrfError::new_unknown_language_error)?,
      link_policy: LinkPolicyArgument::of(matches),
    };

    // A single source keeps working through the path-taking reader: a VFS mounts a directory, never a
    // file, and one source with no mounted roots is the case that convenience exists for.
    let result: TranslationBuildResult = if paths.len() == 1 && paths[0].is_file() {
      xrf_output::info!(
        options.output,
        "Building translation {}, language - {}, sorted - {}",
        format_path(&paths[0]),
        language,
        is_sorted
      );

      TranslationBuilder::build_file(paths[0], &options)?
    } else {
      let source: XrayMountMode = XrayRootsArguments::source_of(matches)?;
      let roots: XrayRoots = XrayRootsArguments::of(matches)?;

      xrf_output::info!(
        options.output,
        "Building translations in {} ({source}), language - {}, sorted - {}",
        roots.describe(),
        language,
        is_sorted
      );

      TranslationBuilder::build_roots(&roots, XrayRootsArguments::prefix_of(matches), &options)?
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
  use crate::core::command::GenericCommand;

  fn parse_matches(extra: &[&str]) -> ArgMatches {
    let mut arguments = vec!["build", "--path", "translations", "--dest", "output"];

    arguments.extend_from_slice(extra);

    BuildCommand.init().try_get_matches_from(arguments).unwrap()
  }

  #[test]
  fn translations_are_sorted_unless_source_order_is_requested() {
    assert!(parse_matches(&[]).get_flag("sort"));
    assert!(!parse_matches(&["--no-sort"]).get_flag("sort"));
  }
}
