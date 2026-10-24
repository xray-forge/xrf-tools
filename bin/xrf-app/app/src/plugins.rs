//! Which plugins the application assembles itself from.

use tauri::Runtime;
use tauri::plugin::TauriPlugin;
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

/// The plugins this application registers, set up in this order: render first, since a domain's setup takes the units
/// it manages, then core's own plugins and every domain.
pub(crate) fn domain_plugins<R: Runtime>() -> Vec<TauriPlugin<R>> {
  vec![
    RenderPlugin::init(),
    AssetsPlugin::init(),
    ArchivesPlugin::init(),
    ConfigsPlugin::init(),
    DialogsPlugin::init(),
    ExportsPlugin::init(),
    GamedataPlugin::init(),
    JobsPlugin::init(),
    LevelsPlugin::init(),
    SessionsPlugin::init(),
    SpawnPlugin::init(),
    SpriteEquipmentPlugin::init(),
    SystemPlugin::init(),
    TexturesPlugin::init(),
    TranslationsPlugin::init(),
    TransportPlugin::init(),
    VisualsPlugin::init(),
  ]
}
