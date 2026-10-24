use crate::commands::dds;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `dds` domain and the operations it exposes.
pub(crate) struct DdsGroup;

impl DdsGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "dds",
      label: "DDS",
      about: "DDS texture file tools",
      commands: vec![
        dds::convert::ConvertCommand::new_box(),
        dds::crop::CropCommand::new_box(),
        dds::info::InfoCommand::new_box(),
        dds::make_bump::MakeBumpCommand::new_box(),
      ],
    }
  }
}
