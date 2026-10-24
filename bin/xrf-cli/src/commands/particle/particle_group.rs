use xrf_particles::ParticlesFile;

use crate::commands::{packed_file, particle};
use crate::core::command::{CommandGroup, GenericCommand};

/// The `particle` domain and the operations it exposes.
pub(crate) struct ParticleGroup;

impl ParticleGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "particle",
      label: "Particle",
      about: "Particle library tools",
      commands: vec![
        particle::find::FindCommand::new_box(),
        particle::info::InfoCommand::new_box(),
        particle::inspect::InspectCommand::new_box(),
        particle::list::ListCommand::new_box(),
        packed_file::PackCommand::<ParticlesFile>::new_box(),
        packed_file::RepackCommand::<ParticlesFile>::new_box(),
        packed_file::ReUnpackCommand::<ParticlesFile>::new_box(),
        packed_file::UnpackCommand::<ParticlesFile>::new_box(),
        packed_file::VerifyCommand::<ParticlesFile>::new_box(),
      ],
    }
  }
}
