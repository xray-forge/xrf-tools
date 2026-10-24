//! Every Tauri plugin whose commands are mirrored into the frontend.

use xrf_app_archives::plugin::ArchivesPlugin;
use xrf_app_configs::plugin::ConfigsPlugin;
use xrf_app_core::plugins::assets::plugin::AssetsPlugin;
use xrf_app_core::plugins::jobs::plugin::JobsPlugin;
use xrf_app_core::plugins::sessions::plugin::SessionsPlugin;
use xrf_app_core::plugins::transport::plugin::TransportPlugin;
use xrf_app_dialogs::plugin::DialogsPlugin;
use xrf_app_exports::plugin::ExportsPlugin;
use xrf_app_gamedata::plugin::GamedataPlugin;
use xrf_app_levels::plugin::LevelsPlugin;
use xrf_app_render::plugin::RenderPlugin;
use xrf_app_spawn::plugin::SpawnPlugin;
use xrf_app_sprite_equipment::plugin::SpriteEquipmentPlugin;
use xrf_app_system::plugin::SystemPlugin;
use xrf_app_textures::plugin::TexturesPlugin;
use xrf_app_translations::plugin::TranslationsPlugin;
use xrf_app_visuals::plugin::VisualsPlugin;
use xrf_ipc_typescript::IpcCommandSurface;

/// The command surfaces this application exports, in the order their modules are written.
///
/// A plugin absent from this list is absent from the frontend, which is why the list is written out rather than
/// derived: it is meant to be read against the plugin registrations, not generated from them.
pub(crate) fn command_surfaces<R: tauri::Runtime>() -> Vec<IpcCommandSurface<R>> {
  vec![
    IpcCommandSurface::new(
      AssetsPlugin::NAME,
      AssetsPlugin::specta_builder::<R>(),
      xrf_app_core::registry::assets::RAW_COMMANDS,
      xrf_app_core::registry::assets::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      ArchivesPlugin::NAME,
      ArchivesPlugin::specta_builder::<R>(),
      xrf_app_archives::registry::archives::RAW_COMMANDS,
      xrf_app_archives::registry::archives::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      ConfigsPlugin::NAME,
      ConfigsPlugin::specta_builder::<R>(),
      xrf_app_configs::registry::configs::RAW_COMMANDS,
      xrf_app_configs::registry::configs::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      DialogsPlugin::NAME,
      DialogsPlugin::specta_builder::<R>(),
      xrf_app_dialogs::registry::dialogs::RAW_COMMANDS,
      xrf_app_dialogs::registry::dialogs::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      ExportsPlugin::NAME,
      ExportsPlugin::specta_builder::<R>(),
      xrf_app_exports::registry::exports::RAW_COMMANDS,
      xrf_app_exports::registry::exports::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      GamedataPlugin::NAME,
      GamedataPlugin::specta_builder::<R>(),
      xrf_app_gamedata::registry::gamedata::RAW_COMMANDS,
      xrf_app_gamedata::registry::gamedata::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      JobsPlugin::NAME,
      JobsPlugin::specta_builder::<R>(),
      xrf_app_core::registry::jobs::RAW_COMMANDS,
      xrf_app_core::registry::jobs::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      LevelsPlugin::NAME,
      LevelsPlugin::specta_builder::<R>(),
      xrf_app_levels::registry::levels::RAW_COMMANDS,
      xrf_app_levels::registry::levels::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      RenderPlugin::NAME,
      RenderPlugin::specta_builder::<R>(),
      xrf_app_render::registry::render::RAW_COMMANDS,
      xrf_app_render::registry::render::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      SessionsPlugin::NAME,
      SessionsPlugin::specta_builder::<R>(),
      xrf_app_core::registry::sessions::RAW_COMMANDS,
      xrf_app_core::registry::sessions::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      SpawnPlugin::NAME,
      SpawnPlugin::specta_builder::<R>(),
      xrf_app_spawn::registry::spawn::RAW_COMMANDS,
      xrf_app_spawn::registry::spawn::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      SpriteEquipmentPlugin::NAME,
      SpriteEquipmentPlugin::specta_builder::<R>(),
      xrf_app_sprite_equipment::registry::sprite_equipment::RAW_COMMANDS,
      xrf_app_sprite_equipment::registry::sprite_equipment::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      SystemPlugin::NAME,
      SystemPlugin::specta_builder::<R>(),
      xrf_app_system::registry::system::RAW_COMMANDS,
      xrf_app_system::registry::system::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      TexturesPlugin::NAME,
      TexturesPlugin::specta_builder::<R>(),
      xrf_app_textures::registry::textures::RAW_COMMANDS,
      xrf_app_textures::registry::textures::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      TranslationsPlugin::NAME,
      TranslationsPlugin::specta_builder::<R>(),
      xrf_app_translations::registry::translations::RAW_COMMANDS,
      xrf_app_translations::registry::translations::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      TransportPlugin::NAME,
      TransportPlugin::specta_builder::<R>(),
      xrf_app_core::registry::transport::RAW_COMMANDS,
      xrf_app_core::registry::transport::BULK_ROUTES,
    ),
    IpcCommandSurface::new(
      VisualsPlugin::NAME,
      VisualsPlugin::specta_builder::<R>(),
      xrf_app_visuals::registry::visuals::RAW_COMMANDS,
      xrf_app_visuals::registry::visuals::BULK_ROUTES,
    ),
  ]
}
