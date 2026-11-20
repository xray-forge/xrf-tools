use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_spawn_builder::{SpawnRootGamedata, SpawnRootOptions, SpawnRootOutcome, SpawnRootRequest};
use xrf_utils::format_path;

use crate::core::arguments::{InputArgument, JobsArgument, LtxDialectArgument};
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Builds a root spawn from a gamedata root's levels, as SDK 0.7's `xrAI -s` does.
#[derive(Default)]
pub(crate) struct BuildCommand;

impl GenericCommand for BuildCommand {
  fn operation(&self) -> &'static str {
    "build"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path(
      "Gamedata root holding configs/game_levels.ltx, and by default the levels' directories under levels/ and \
             the spawn written under spawns/",
    )
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about(
        "Build a root spawn from a gamedata root's levels, as SDK 0.7's xrAI -s does: each level's level.ai, \
         level.spawn and level.game into one spawn with the game graph, its cross tables, the entities, artefact \
         points and patrols. The same levels give the same file",
      )
      .arg(
        Arg::new("levels-root")
          .help("Directory holding the levels' directories; by default levels/ under the gamedata root")
          .long("levels-root")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("The .spawn file to write; by default spawns/all.spawn under the gamedata root, the game's own")
          .short('d')
          .long("dest")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("levels")
          .help(
            "Levels to build, by directory name, comma-separated or repeated; by default every level the game lists \
             whose directory holds level.ai, level.spawn and level.game",
          )
          .long("levels")
          .action(ArgAction::Append)
          .value_delimiter(',')
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("start")
          .help(
            "Level the actor starts on, where it is not the actor's own: it moves to that level's first game vertex",
          )
          .long("start")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("strict-separators")
          .help(
            "Refuse the build where a restrictor flagged for the separator check cuts an AI map apart, rather than \
             report it",
          )
          .long("strict-separators")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("force")
          .help("Overwrite the spawn where its directory already holds it, the name compared without case")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      );

    LtxDialectArgument::declare(JobsArgument::declare(command))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let (destination, name) = Self::split_destination(matches.get_one::<PathBuf>("dest"))?;
    let request: SpawnRootRequest = SpawnRootRequest {
      gamedata: InputArgument::of(matches).clone(),
      levels_root: matches.get_one::<PathBuf>("levels-root").cloned(),
      levels: matches
        .get_many::<String>("levels")
        .map(|levels| levels.cloned().collect()),
      destination,
      name,
      options: SpawnRootOptions {
        start: matches.get_one::<String>("start").cloned(),
        is_separator_strict: matches.get_flag("strict-separators"),
      },
      is_forced: matches.get_flag("force"),
    };
    let outcome: SpawnRootOutcome = SpawnRootGamedata::build(
      &request,
      LtxDialectArgument::of(matches).as_ref(),
      &CliJob::for_output(context.get_output()),
    )?;

    Self::print(context.get_output(), &outcome);
    context.set_result(|| &outcome)?;

    Ok(())
  }
}

impl BuildCommand {
  /// The spawn the game loads, written where no destination is named.
  const DEFAULT_NAME: &'static str = "all";

  /// The directory and name of the `.spawn` file `dest` names: none and the game's own name where it names none.
  ///
  /// # Errors
  ///
  /// Returns an error where `dest` does not name a `.spawn` file.
  fn split_destination(dest: Option<&PathBuf>) -> XrfResult<(Option<PathBuf>, String)> {
    let Some(dest) = dest else {
      return Ok((None, String::from(Self::DEFAULT_NAME)));
    };
    let name: Option<&str> = dest
      .extension()
      .filter(|extension| extension.eq_ignore_ascii_case("spawn"))
      .and_then(|_| dest.file_stem())
      .and_then(|stem| stem.to_str())
      .filter(|stem| !stem.is_empty());
    let Some(name) = name else {
      return Err(XrfError::new_invalid_error(format!(
        "{} names no .spawn file to write",
        format_path(dest)
      )));
    };
    let directory: PathBuf = dest
      .parent()
      .filter(|parent| !parent.as_os_str().is_empty())
      .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    Ok((Some(directory), String::from(name)))
  }

  /// What the build found, then what it wrote.
  fn print(output: &OutputOptions, outcome: &SpawnRootOutcome) {
    for check in outcome.report.checks() {
      for finding in check.findings() {
        xrf_output::warning!(
          output,
          "{}: {}",
          finding.subject().unwrap_or_default(),
          finding.message()
        );
      }
    }

    xrf_output::success!(
      output,
      "Built {} from {}: {} objects, {} artefact points, {} patrols, {} warnings",
      outcome.path,
      outcome.levels.join(", "),
      outcome.objects,
      outcome.artefact_points,
      outcome.patrols,
      outcome.report.count_findings()
    );
  }
}
