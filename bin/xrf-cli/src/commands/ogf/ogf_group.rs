use crate::commands::ogf;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `ogf` domain and the operations it exposes.
pub(crate) struct OgfGroup;

impl OgfGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
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
    }
  }
}
