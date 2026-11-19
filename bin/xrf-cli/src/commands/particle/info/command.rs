use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_particles::ParticlesFile;
use xrf_utils::format_path;

use super::report::ParticleInfoReport;
use crate::commands::particle::particles_file_input::ParticlesFileInput;
use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn input(&self) -> CommandInput {
    ParticlesFileInput::input()
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Print what a particle file holds")
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read particle file {}", format_path(InputArgument::of(matches)));

    let particles_file: ParticlesFile = ParticlesFileInput::open(matches)?;

    xrf_output::info!(output, "Particles file information:");

    xrf_output::info!(output, "Version: {}", particles_file.header.version);
    xrf_output::info!(output, "Effects count: {}", particles_file.effects.effects.len());
    xrf_output::info!(output, "Groups count: {}", particles_file.groups.groups.len());

    context.set_result(|| ParticleInfoReport::new(&particles_file))?;

    Ok(())
  }
}
