use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_level_compiler::{SpawnRootGamedata, SpawnRootOptions, SpawnRootOutcome, SpawnRootRequest};
use xrf_output::OutputOptions;

use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Builds a root spawn from a gamedata root's levels, as SDK 0.7's `xrAI -s` does.
#[derive(Default)]
pub(crate) struct BuildCommand;

impl GenericCommand for BuildCommand {
  fn operation(&self) -> &'static str {
    "build"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Build a root spawn from a gamedata root's levels, as SDK 0.7's xrAI -s does: each level's level.ai, \
         level.spawn and level.game into one spawn with the game graph, its cross tables, the entities, artefact \
         points and patrols, written as `spawns/<out>.spawn`. The same levels give the same file",
      )
      .arg(
        Arg::new("path")
          .help("Gamedata root holding configs/game_levels.ltx and the levels' directories under levels/")
          .short('p')
          .long("path")
          .required(true)
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
        Arg::new("out")
          .help("Name of the spawn to write, `spawns/<out>.spawn`; `all` is the game's own")
          .long("out")
          .required(true)
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
          .help("Write over spawns/all.spawn, the game's own spawn, where it is there")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let request: SpawnRootRequest = SpawnRootRequest {
      gamedata: matches
        .get_one::<PathBuf>("path")
        .expect("the path is required")
        .clone(),
      levels: matches
        .get_many::<String>("levels")
        .map(|levels| levels.cloned().collect()),
      name: matches.get_one::<String>("out").expect("the name is required").clone(),
      options: SpawnRootOptions {
        start: matches.get_one::<String>("start").cloned(),
        is_separator_strict: matches.get_flag("strict-separators"),
      },
      is_forced: matches.get_flag("force"),
    };
    let outcome: SpawnRootOutcome = SpawnRootGamedata::build(&request)?;

    Self::print(context.get_output(), &outcome);
    context.set_result(|| &outcome)?;

    Ok(())
  }
}

impl BuildCommand {
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
