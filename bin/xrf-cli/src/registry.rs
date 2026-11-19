use crate::commands::archive::ArchiveGroup;
use crate::commands::dds::DdsGroup;
use crate::commands::dialog::DialogGroup;
use crate::commands::docs::DocsGroup;
use crate::commands::externs::ExternsGroup;
use crate::commands::gamedata::GamedataGroup;
use crate::commands::gamemtl::GameMtlGroup;
use crate::commands::level::LevelGroup;
use crate::commands::ltx::LtxGroup;
use crate::commands::object::ObjectGroup;
use crate::commands::ogf::OgfGroup;
use crate::commands::omf::OmfGroup;
use crate::commands::particle::ParticleGroup;
use crate::commands::spawn::SpawnGroup;
use crate::commands::sprite::SpriteGroup;
use crate::commands::thm::ThmGroup;
use crate::commands::translation::TranslationGroup;
use crate::core::command::CommandGroup;

impl CommandGroup {
  /// Every public CLI domain, each declaring its own operations, for dispatch and for `docs generate`.
  pub(crate) fn list_all() -> Vec<Self> {
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
      ObjectGroup::declare(),
      OgfGroup::declare(),
      OmfGroup::declare(),
      ParticleGroup::declare(),
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

  use clap::{Arg, Command};

  use crate::core::arguments::InputArgument;
  use crate::core::command::CommandGroup;

  // todo: Move these domains' inputs to `InputArgument` and their positional selectors to named flags, then drop
  // the list so the rule below holds for every command.
  /// Domains still reading their input through `-p/--path`.
  const NAMED_INPUT_DOMAINS: [&str; 2] = ["level", "ltx"];

  /// What a command reads is its one positional, and nothing names it `-p/--path`.
  #[test]
  fn every_command_reads_its_input_positionally() {
    for group in CommandGroup::list_all() {
      for command in &group.commands {
        let declared: Command = command.init();
        let name: String = format!("{} {}", group.slug, command.operation());
        let positionals: Vec<&Arg> = declared.get_positionals().collect();

        assert!(positionals.len() <= 1, "'{name}' declares more than one positional");

        if NAMED_INPUT_DOMAINS.contains(&group.slug) {
          continue;
        }

        assert!(
          positionals
            .iter()
            .all(|argument| argument.get_id() == InputArgument::ID),
          "'{name}' declares a positional that is not its input"
        );
        assert!(
          declared
            .get_arguments()
            .all(|argument| argument.get_long() != Some("path") && argument.get_short() != Some('p')),
          "'{name}' still names an input with -p/--path"
        );
      }
    }
  }

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
