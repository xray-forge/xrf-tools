use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_particles::ParticlesFile;
use xrf_utils::format_path;

use super::report::ParticleInfoReport;
use crate::commands::particle::particles_file_arguments::ParticlesFileArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    ParticlesFileArguments::declare(command.about("Print what a particle file holds"))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Read particle file {}",
      format_path(ParticlesFileArguments::path_of(matches))
    );

    let particles_file: ParticlesFile = ParticlesFileArguments::open(matches)?;

    xrf_output::info!(output, "Particles file information:");

    xrf_output::info!(output, "Version: {}", particles_file.header.version);
    xrf_output::info!(output, "Effects count: {}", particles_file.effects.effects.len());
    xrf_output::info!(output, "Groups count: {}", particles_file.groups.groups.len());

    context.set_result(|| ParticleInfoReport::new(&particles_file))?;

    Ok(())
  }
}
