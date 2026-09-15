use std::collections::HashSet;

use serde::Serialize;

use crate::plugins::archives::describe::archive_reference::ArchiveReferenceStatus;
use crate::plugins::archives::describe::particles::archive_particles_effect::ArchiveParticlesEffect;
use crate::plugins::archives::describe::particles::archive_particles_group::ArchiveParticlesGroup;

/// What the library holds, taken over the whole of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesLibrary {
  pub version: u16,
  pub effects: usize,
  pub groups: usize,
  /// Actions across every effect.
  pub actions: usize,
  /// Distinct textures the effects draw from, which is far fewer than the effects naming them - 125 against 921 in
  /// vanilla.
  pub textures: usize,
  /// Distinct textures the subject being browsed does not hold.
  pub absent_textures: usize,
  /// Effect names the groups use that this library does not define.
  pub undefined_effects: usize,
}

impl ArchiveParticlesLibrary {
  /// The totals of a library, taken off what was already described so the two cannot disagree.
  pub fn of(version: u16, effects: &[ArchiveParticlesEffect], groups: &[ArchiveParticlesGroup]) -> Self {
    let mut textures: HashSet<&str> = HashSet::new();
    let mut absent: HashSet<&str> = HashSet::new();

    for effect in effects {
      textures.insert(effect.texture.name.as_str());

      if effect.texture.status != ArchiveReferenceStatus::Present {
        absent.insert(effect.texture.name.as_str());
      }
    }

    Self {
      version,
      effects: effects.len(),
      groups: groups.len(),
      actions: effects.iter().map(|effect| effect.actions_count).sum(),
      textures: textures.len(),
      absent_textures: absent.len(),
      undefined_effects: Self::count_undefined(groups),
    }
  }

  /// Distinct effect names the groups reach for and the library does not define, child slots included.
  fn count_undefined(groups: &[ArchiveParticlesGroup]) -> usize {
    let mut undefined: HashSet<&str> = HashSet::new();

    for group in groups {
      for slot in &group.effects {
        for named in [
          Some(&slot.effect),
          slot.on_birth.as_ref(),
          slot.on_play.as_ref(),
          slot.on_dead.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
          if !named.is_defined {
            undefined.insert(named.name.as_str());
          }
        }
      }
    }

    undefined.len()
  }
}
