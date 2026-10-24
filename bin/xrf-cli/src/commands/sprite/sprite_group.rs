use crate::commands::sprite;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `sprite` domain and the operations it exposes.
pub(crate) struct SpriteGroup;

impl SpriteGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
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
    }
  }
}
