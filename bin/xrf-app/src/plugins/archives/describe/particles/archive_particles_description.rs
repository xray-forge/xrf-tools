use std::collections::HashSet;

use serde::Serialize;
use xrf_db::{ParticlesFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::particles::archive_particles_effect::ArchiveParticlesEffect;
use crate::plugins::archives::describe::particles::archive_particles_group::ArchiveParticlesGroup;
use crate::plugins::archives::describe::particles::archive_particles_library::ArchiveParticlesLibrary;

/// Everything the viewer says about the particle library.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesDescription {
  pub library: ArchiveParticlesLibrary,
  pub effects: Vec<ArchiveParticlesEffect>,
  pub groups: Vec<ArchiveParticlesGroup>,
}

impl ArchiveParticlesDescription {
  /// Reads the library an entry holds and resolves the textures it draws from.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a particle library this reader can
  /// walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: ParticlesFile = ParticlesFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    let effects: Vec<ArchiveParticlesEffect> = ArchiveParticlesEffect::of_all(source, &file.effects.effects);
    let defined: HashSet<&str> = file.effects.effects.iter().map(|effect| effect.name.as_str()).collect();
    let groups: Vec<ArchiveParticlesGroup> = ArchiveParticlesGroup::of_all(&file.groups.groups, &defined);

    Ok(Self {
      library: ArchiveParticlesLibrary::of(file.header.version, &effects, &groups),
      effects,
      groups,
    })
  }
}
