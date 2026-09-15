//! Which command domains the application assembles itself from.

use tauri::Runtime;
use tauri::plugin::TauriPlugin;

use crate::plugins::archives::plugin::ArchivesPlugin;
use crate::plugins::assets::plugin::AssetsPlugin;
use crate::plugins::configs::plugin::ConfigsPlugin;
use crate::plugins::dialogs::plugin::DialogsPlugin;
use crate::plugins::exports::plugin::ExportsPlugin;
use crate::plugins::gamedata::plugin::GamedataPlugin;
use crate::plugins::jobs::plugin::JobsPlugin;
use crate::plugins::spawn::plugin::SpawnPlugin;
use crate::plugins::sprite_equipment::plugin::SpriteEquipmentPlugin;
use crate::plugins::system::plugin::SystemPlugin;
use crate::plugins::textures::plugin::TexturesPlugin;
use crate::plugins::translations::plugin::TranslationsPlugin;
use crate::plugins::visuals::plugin::VisualsPlugin;

/// The domain plugins this application registers, in the order their command surfaces are written.
pub(crate) fn domain_plugins<R: Runtime>() -> Vec<TauriPlugin<R>> {
  vec![
    AssetsPlugin::init(),
    ArchivesPlugin::init(),
    ConfigsPlugin::init(),
    DialogsPlugin::init(),
    ExportsPlugin::init(),
    GamedataPlugin::init(),
    JobsPlugin::init(),
    SpawnPlugin::init(),
    SpriteEquipmentPlugin::init(),
    SystemPlugin::init(),
    TexturesPlugin::init(),
    TranslationsPlugin::init(),
    VisualsPlugin::init(),
  ]
}
