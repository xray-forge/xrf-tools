use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_particles::ParticlesFile;
use xrf_utils::format_path;

use super::report::ParticleInfoReport;
use crate::commands::particle::particles_file_open::{
  ParticlesFileArguments, open_particles_file, requested_particles_path,
};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  /// Create command for printing particle file info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about provided particle file")
      .with_particles_file()
  }

  /// Print information about particle file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Read particle file {}",
      format_path(requested_particles_path(matches))
    );

    let particles_file: Box<ParticlesFile> = Box::new(open_particles_file(matches)?);

    xrf_output::info!(output, "Particles file information:");

    xrf_output::info!(output, "Version: {}", particles_file.header.version);
    xrf_output::info!(output, "Effects count: {}", particles_file.effects.effects.len());
    xrf_output::info!(output, "Groups count: {}", particles_file.groups.groups.len());

    context.set_result(|| ParticleInfoReport::new(&particles_file))?;

    Ok(())
  }
}
