//! Every Tauri plugin whose commands are mirrored into the frontend.

use xrf_ipc_typescript::IpcCommandSurface;

use crate::plugins::archives::plugin::ArchivesPlugin;
use crate::plugins::assets::plugin::AssetsPlugin;
use crate::plugins::configs::plugin::ConfigsPlugin;
use crate::plugins::dialogs::plugin::DialogsPlugin;
use crate::plugins::exports::plugin::ExportsPlugin;
use crate::plugins::gamedata::plugin::GamedataPlugin;
use crate::plugins::jobs::plugin::JobsPlugin;
use crate::plugins::levels::plugin::LevelsPlugin;
use crate::plugins::spawn::plugin::SpawnPlugin;
use crate::plugins::sprite_equipment::plugin::SpriteEquipmentPlugin;
use crate::plugins::system::plugin::SystemPlugin;
use crate::plugins::textures::plugin::TexturesPlugin;
use crate::plugins::translations::plugin::TranslationsPlugin;
use crate::plugins::visuals::plugin::VisualsPlugin;

/// The command surfaces this application exports, in the order their modules are written.
///
/// A plugin absent from this list is absent from the frontend, which is why the list is written out rather than
/// derived: it is meant to be read against the plugin registrations, not generated from them.
pub(crate) fn command_surfaces<R: tauri::Runtime>() -> Vec<IpcCommandSurface<R>> {
  vec![
    IpcCommandSurface::new(
      AssetsPlugin::NAME,
      AssetsPlugin::specta_builder::<R>(),
      crate::ipc::registry::assets::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      ArchivesPlugin::NAME,
      ArchivesPlugin::specta_builder::<R>(),
      crate::ipc::registry::archives::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      ConfigsPlugin::NAME,
      ConfigsPlugin::specta_builder::<R>(),
      crate::ipc::registry::configs::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      DialogsPlugin::NAME,
      DialogsPlugin::specta_builder::<R>(),
      crate::ipc::registry::dialogs::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      ExportsPlugin::NAME,
      ExportsPlugin::specta_builder::<R>(),
      crate::ipc::registry::exports::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      GamedataPlugin::NAME,
      GamedataPlugin::specta_builder::<R>(),
      crate::ipc::registry::gamedata::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      JobsPlugin::NAME,
      JobsPlugin::specta_builder::<R>(),
      crate::ipc::registry::jobs::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      LevelsPlugin::NAME,
      LevelsPlugin::specta_builder::<R>(),
      crate::ipc::registry::levels::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      SpawnPlugin::NAME,
      SpawnPlugin::specta_builder::<R>(),
      crate::ipc::registry::spawn::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      SpriteEquipmentPlugin::NAME,
      SpriteEquipmentPlugin::specta_builder::<R>(),
      crate::ipc::registry::sprite_equipment::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      SystemPlugin::NAME,
      SystemPlugin::specta_builder::<R>(),
      crate::ipc::registry::system::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      TexturesPlugin::NAME,
      TexturesPlugin::specta_builder::<R>(),
      crate::ipc::registry::textures::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      TranslationsPlugin::NAME,
      TranslationsPlugin::specta_builder::<R>(),
      crate::ipc::registry::translations::RAW_COMMANDS,
    ),
    IpcCommandSurface::new(
      VisualsPlugin::NAME,
      VisualsPlugin::specta_builder::<R>(),
      crate::ipc::registry::visuals::RAW_COMMANDS,
    ),
  ]
}
