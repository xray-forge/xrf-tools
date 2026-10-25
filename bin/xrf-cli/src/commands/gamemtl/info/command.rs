use clap::{ArgMatches, Command};
use xrf_gamemtl::GameMtlFile;
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::GameMtlInfoReport;
use crate::commands::gamemtl::gamemtl_file_arguments::GameMtlFileArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    GameMtlFileArguments::declare(command.about("Print what a game material library holds"))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Read game material library {}",
      format_path(GameMtlFileArguments::path_of(matches))
    );

    let file: GameMtlFile = GameMtlFileArguments::open(matches)?;

    xrf_output::info!(output, "Game material library information:");

    xrf_output::info!(output, "Version: {}", file.version);
    xrf_output::info!(output, "Game: {}", file.get_game_version());
    xrf_output::info!(
      output,
      "Materials count: {}, dynamic: {}",
      file.materials.len(),
      file.get_dynamic_materials_count()
    );
    xrf_output::info!(
      output,
      "Pairs count: {}, inheriting: {}",
      file.pairs.len(),
      file.get_inheriting_pairs_count()
    );

    context.set_result(|| GameMtlInfoReport::new(&file))?;

    Ok(())
  }
}
