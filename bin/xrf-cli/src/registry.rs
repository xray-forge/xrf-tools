use crate::commands::archive::pack_archive::PackArchiveCommand;
use crate::commands::archive::unpack_archive::UnpackArchiveCommand;
use crate::commands::assets::list_assets::ListAssetsCommand;
use crate::commands::dialog::parse_dialog::ParseDialogCommand;
use crate::commands::docs::generate_docs::GenerateDocsCommand;
use crate::commands::externs::export_externs::ExportExternsCommand;
use crate::commands::gamedata::verify_gamedata::VerifyGamedataCommand;
use crate::commands::ltx::format_ltx::FormatLtxCommand;
use crate::commands::ltx::verify_ltx::VerifyLtxCommand;
use crate::commands::ogf::info_ogf::InfoOgfCommand;
use crate::commands::ogf::patch_ogf_motion_refs::PatchOgfMotionRefsCommand;
use crate::commands::ogf::patch_ogf_texture_refs::PatchOgfTextureRefsCommand;
use crate::commands::ogf::verify_ogf::VerifyOgfCommand;
use crate::commands::omf::duplicate_omf_motion::DuplicateOmfMotionCommand;
use crate::commands::omf::filter_omf_motions::FilterOmfMotionsCommand;
use crate::commands::omf::info_omf::InfoOmfCommand;
use crate::commands::omf::rename_omf_motions::RenameOmfMotionsCommand;
use crate::commands::omf::repack_omf::RepackOmfCommand;
use crate::commands::particle::info_particles::InfoParticlesCommand;
use crate::commands::particle::pack_particles::PackParticlesCommand;
use crate::commands::particle::re_unpack_particles::ReUnpackParticlesCommand;
use crate::commands::particle::repack_particles::RepackParticlesCommand;
use crate::commands::particle::unpack_particles::UnpackParticlesCommand;
use crate::commands::particle::verify_particles::VerifyParticlesCommand;
use crate::commands::spawn::info_spawn::InfoSpawnCommand;
use crate::commands::spawn::pack_spawn::PackSpawnCommand;
use crate::commands::spawn::repack_spawn::RepackSpawnCommand;
use crate::commands::spawn::unpack_spawn::UnpackSpawnCommand;
use crate::commands::spawn::verify_spawn::VerifySpawnCommand;
use crate::commands::texture::crop_dds::CropDdsCommand;
use crate::commands::texture::info_dds::InfoDdsCommand;
use crate::commands::texture::pack_equipment_icons::PackEquipmentIconsCommand;
use crate::commands::texture::pack_texture_description::PackTextureDescriptionCommand;
use crate::commands::texture::unpack_equipment_icons::UnpackEquipmentIconsCommand;
use crate::commands::texture::unpack_texture_description::UnpackTextureDescriptionCommand;
use crate::commands::texture::verify_equipment_icons::VerifyEquipmentIconsCommand;
use crate::commands::thm::patch_thm_bump::PatchThmBumpCommand;
use crate::commands::translation::build_translation::BuildTranslationCommand;
use crate::commands::translation::initialize_translation::InitializeTranslationCommand;
use crate::commands::translation::parse_translation::ParseTranslationCommand;
use crate::commands::translation::verify_translation::VerifyTranslationCommand;
use crate::core::generic_command::{CommandGroup, GenericCommand};

pub fn setup_command_groups() -> Vec<CommandGroup> {
  vec![
    CommandGroup {
      name: "Archive",
      commands: vec![PackArchiveCommand::new_box(), UnpackArchiveCommand::new_box()],
    },
    CommandGroup {
      name: "Assets",
      commands: vec![ListAssetsCommand::new_box()],
    },
    CommandGroup {
      name: "Dialog",
      commands: vec![ParseDialogCommand::new_box()],
    },
    CommandGroup {
      name: "Docs",
      commands: vec![GenerateDocsCommand::new_box()],
    },
    CommandGroup {
      name: "Externs",
      commands: vec![ExportExternsCommand::new_box()],
    },
    CommandGroup {
      name: "Gamedata",
      commands: vec![VerifyGamedataCommand::new_box()],
    },
    CommandGroup {
      name: "LTX",
      commands: vec![FormatLtxCommand::new_box(), VerifyLtxCommand::new_box()],
    },
    CommandGroup {
      name: "OGF",
      commands: vec![
        InfoOgfCommand::new_box(),
        PatchOgfMotionRefsCommand::new_box(),
        PatchOgfTextureRefsCommand::new_box(),
        VerifyOgfCommand::new_box(),
      ],
    },
    CommandGroup {
      name: "OMF",
      commands: vec![
        DuplicateOmfMotionCommand::new_box(),
        FilterOmfMotionsCommand::new_box(),
        InfoOmfCommand::new_box(),
        RenameOmfMotionsCommand::new_box(),
        RepackOmfCommand::new_box(),
      ],
    },
    CommandGroup {
      name: "Particle",
      commands: vec![
        InfoParticlesCommand::new_box(),
        PackParticlesCommand::new_box(),
        RepackParticlesCommand::new_box(),
        ReUnpackParticlesCommand::new_box(),
        UnpackParticlesCommand::new_box(),
        VerifyParticlesCommand::new_box(),
      ],
    },
    CommandGroup {
      name: "Spawn",
      commands: vec![
        InfoSpawnCommand::new_box(),
        PackSpawnCommand::new_box(),
        RepackSpawnCommand::new_box(),
        UnpackSpawnCommand::new_box(),
        VerifySpawnCommand::new_box(),
      ],
    },
    CommandGroup {
      name: "Texture",
      commands: vec![
        CropDdsCommand::new_box(),
        InfoDdsCommand::new_box(),
        PackEquipmentIconsCommand::new_box(),
        PackTextureDescriptionCommand::new_box(),
        UnpackEquipmentIconsCommand::new_box(),
        VerifyEquipmentIconsCommand::new_box(),
        UnpackTextureDescriptionCommand::new_box(),
      ],
    },
    CommandGroup {
      name: "THM",
      commands: vec![PatchThmBumpCommand::new_box()],
    },
    CommandGroup {
      name: "Translation",
      commands: vec![
        BuildTranslationCommand::new_box(),
        InitializeTranslationCommand::new_box(),
        ParseTranslationCommand::new_box(),
        VerifyTranslationCommand::new_box(),
      ],
    },
  ]
}

pub fn setup_subcommands() -> Vec<Box<dyn GenericCommand>> {
  setup_command_groups()
    .into_iter()
    .flat_map(|group| group.commands)
    .collect()
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use super::setup_subcommands;

  #[test]
  fn registered_command_names_are_unique() {
    let mut names: HashSet<&'static str> = HashSet::new();

    for command in setup_subcommands() {
      assert!(
        names.insert(command.name()),
        "Duplicated command name '{}'",
        command.name()
      );
    }
  }
}
