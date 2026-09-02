use crate::commands::{
  archive, dds, dialog, docs, externs, gamedata, ltx, ogf, omf, particle, profile, spawn, sprite, thm, translation,
};
use crate::core::generic_command::{CommandGroup, GenericCommand};

/// Registers every public CLI domain and the operations it exposes.
pub fn setup_command_groups() -> Vec<CommandGroup> {
  vec![
    CommandGroup {
      slug: "archive",
      label: "Archive",
      about: "X-Ray archive tools",
      commands: vec![
        archive::extract::ExtractCommand::new_box(),
        archive::find::FindCommand::new_box(),
        archive::info::InfoCommand::new_box(),
        archive::list::ListCommand::new_box(),
        archive::pack::PackCommand::new_box(),
        archive::unpack::UnpackCommand::new_box(),
        archive::verify::VerifyCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "dds",
      label: "DDS",
      about: "DDS texture file tools",
      commands: vec![dds::crop::CropCommand::new_box(), dds::info::InfoCommand::new_box()],
    },
    CommandGroup {
      slug: "dialog",
      label: "Dialog",
      about: "Dialog inspection tools",
      commands: vec![dialog::info::InfoCommand::new_box()],
    },
    CommandGroup {
      slug: "docs",
      label: "Docs",
      about: "CLI documentation tools",
      commands: vec![docs::generate::GenerateCommand::new_box()],
    },
    CommandGroup {
      slug: "externs",
      label: "Externs",
      about: "Script extern export tools",
      commands: vec![externs::export::ExportCommand::new_box()],
    },
    CommandGroup {
      slug: "gamedata",
      label: "Gamedata",
      about: "Assembled gamedata tools",
      commands: vec![
        gamedata::list::ListCommand::new_box(),
        gamedata::verify::VerifyCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "ltx",
      label: "LTX",
      about: "LTX configuration tools",
      commands: vec![
        ltx::format::FormatCommand::new_box(),
        ltx::verify::VerifyCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "ogf",
      label: "OGF",
      about: "OGF model tools",
      commands: vec![
        ogf::fix::FixCommand::new_box(),
        ogf::info::InfoCommand::new_box(),
        ogf::patch_motion_refs::PatchMotionRefsCommand::new_box(),
        ogf::patch_texture_refs::PatchTextureRefsCommand::new_box(),
        ogf::verify::VerifyCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "omf",
      label: "OMF",
      about: "OMF motion tools",
      commands: vec![
        omf::duplicate_motion::DuplicateMotionCommand::new_box(),
        omf::filter_motions::FilterMotionsCommand::new_box(),
        omf::info::InfoCommand::new_box(),
        omf::rename_motions::RenameMotionsCommand::new_box(),
        omf::repack::RepackCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "particle",
      label: "Particle",
      about: "Particle library tools",
      commands: vec![
        particle::info::InfoCommand::new_box(),
        particle::pack::PackCommand::new_box(),
        particle::repack::RepackCommand::new_box(),
        particle::re_unpack::ReUnpackCommand::new_box(),
        particle::unpack::UnpackCommand::new_box(),
        particle::verify::VerifyCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "profile",
      label: "Profile",
      about: "Performance measurement tools",
      commands: vec![profile::run::RunCommand::new_box()],
    },
    CommandGroup {
      slug: "spawn",
      label: "Spawn",
      about: "Spawn file tools",
      commands: vec![
        spawn::info::InfoCommand::new_box(),
        spawn::pack::PackCommand::new_box(),
        spawn::repack::RepackCommand::new_box(),
        spawn::unpack::UnpackCommand::new_box(),
        spawn::verify::VerifyCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "sprite",
      label: "Sprite",
      about: "Sprite sheet tools",
      commands: vec![
        sprite::pack_description::PackDescriptionCommand::new_box(),
        sprite::pack_equipment::PackEquipmentCommand::new_box(),
        sprite::unpack_description::UnpackDescriptionCommand::new_box(),
        sprite::unpack_equipment::UnpackEquipmentCommand::new_box(),
        sprite::verify_equipment::VerifyEquipmentCommand::new_box(),
      ],
    },
    CommandGroup {
      slug: "thm",
      label: "THM",
      about: "THM texture metadata tools",
      commands: vec![thm::patch_bump::PatchBumpCommand::new_box()],
    },
    CommandGroup {
      slug: "translation",
      label: "Translation",
      about: "Translation file tools",
      commands: vec![
        translation::build::BuildCommand::new_box(),
        translation::format::FormatCommand::new_box(),
        translation::initialize::InitializeCommand::new_box(),
        translation::parse::ParseCommand::new_box(),
        translation::verify::VerifyCommand::new_box(),
      ],
    },
  ]
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use super::setup_command_groups;

  #[test]
  fn registered_domain_slugs_are_unique() {
    let mut slugs = HashSet::new();

    for group in setup_command_groups() {
      assert!(slugs.insert(group.slug), "Duplicated domain slug '{}'", group.slug);
    }
  }

  #[test]
  fn registered_operation_slugs_are_unique_within_each_domain() {
    for group in setup_command_groups() {
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
