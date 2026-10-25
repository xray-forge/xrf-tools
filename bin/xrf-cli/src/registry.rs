use crate::commands::archive::ArchiveGroup;
use crate::commands::dds::DdsGroup;
use crate::commands::dialog::DialogGroup;
use crate::commands::docs::DocsGroup;
use crate::commands::externs::ExternsGroup;
use crate::commands::gamedata::GamedataGroup;
use crate::commands::gamemtl::GameMtlGroup;
use crate::commands::level::LevelGroup;
use crate::commands::ltx::LtxGroup;
use crate::commands::ogf::OgfGroup;
use crate::commands::omf::OmfGroup;
use crate::commands::particle::ParticleGroup;
use crate::commands::profile::ProfileGroup;
use crate::commands::spawn::SpawnGroup;
use crate::commands::sprite::SpriteGroup;
use crate::commands::thm::ThmGroup;
use crate::commands::translation::TranslationGroup;
use crate::core::command::CommandGroup;

impl CommandGroup {
  /// Every public CLI domain, each declaring its own operations, for dispatch and for `docs generate`.
  pub fn list_all() -> Vec<Self> {
    vec![
      ArchiveGroup::declare(),
      DdsGroup::declare(),
      DialogGroup::declare(),
      DocsGroup::declare(),
      ExternsGroup::declare(),
      GamedataGroup::declare(),
      GameMtlGroup::declare(),
      LevelGroup::declare(),
      LtxGroup::declare(),
      OgfGroup::declare(),
      OmfGroup::declare(),
      ParticleGroup::declare(),
      ProfileGroup::declare(),
      SpawnGroup::declare(),
      SpriteGroup::declare(),
      ThmGroup::declare(),
      TranslationGroup::declare(),
    ]
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use crate::core::command::CommandGroup;

  #[test]
  fn registered_domain_slugs_are_unique() {
    let mut slugs = HashSet::new();

    for group in CommandGroup::list_all() {
      assert!(slugs.insert(group.slug), "Duplicated domain slug '{}'", group.slug);
    }
  }

  #[test]
  fn registered_operation_slugs_are_unique_within_each_domain() {
    for group in CommandGroup::list_all() {
      let mut operations = HashSet::new();

      for command in group.commands {
        assert!(
          operations.insert(command.operation()),
          "Duplicated operation '{}' in '{}'",
          command.operation(),
          group.slug
        );
      }
    }
  }
}
