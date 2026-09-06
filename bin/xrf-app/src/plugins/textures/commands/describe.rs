use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::types::TauriResult;
use crate::plugins::textures::description::TextureDescription;
use crate::plugins::textures::source::TextureSource;

/// Describe one texture: its file, its descriptor as the engine reads it, and the pair the engine binds.
///
/// Resolution happens once, for the texture and both halves, inside one probe, so the three files are looked for in
/// the same roots: a second probe could mount a source between the calls and answer differently. The roots are
/// centred on the file when the source is one, and the effective roots travel back so a later read searches alike.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe"))]
#[tauri::command(rename = "describe")]
pub async fn textures_describe(
  source: TextureSource,
  roots: XrayRoots,
  assets: State<'_, AssetMountState>,
) -> TauriResult<TextureDescription> {
  log::info!("Describing texture: {}", source.label());

  let roots: XrayRoots = roots.centred_on(source.physical_path());
  let description: TextureDescription = assets.with_probe(&roots, |probe| {
    TextureDescription::describe(probe, source, roots.clone())
  })??;

  log::info!(
    "Described texture '{}': {:?}",
    description.reference,
    description.material.as_ref().map(|material| material.outcome)
  );

  Ok(description)
}
