use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;
use xrf_app_render::viewport_shows::ViewportShows;
use xrf_renderer::{RenderLevelSource, RenderViewportId};
use xrf_vfs::XrayRoots;

use crate::description::TextureDescription;
use crate::request::TextureSurfaceRequest;
use crate::surface::render_source::TextureRenderSource;

/// Draw a texture laid on a body in a viewport, its files read by the renderer; no request draws none. Answers the
/// scene drawn, which the viewport's load reports name, or none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_texture"))]
#[tauri::command(rename = "show_texture")]
pub async fn textures_show_texture(
  shows: State<'_, ViewportShows>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  request: Option<TextureSurfaceRequest>,
) -> TauriResult<Option<u32>> {
  // Taken before the description's hop, so a texture asked for later is never replaced by this one.
  let ticket: u64 = shows.ask(viewport);
  let source: Option<Arc<dyn RenderLevelSource>> = match request {
    Some(request) => {
      log::info!(
        "Showing texture '{}' in native viewport {} on a {:?}",
        request.source.label(),
        viewport.0,
        request.shape
      );

      let assets: AssetMountState = AssetMountState::clone(&assets);
      let source: TextureRenderSource = execution
        .run_blocking("Describing a texture surface", move || {
          let roots: XrayRoots = request.roots.centred_on(request.source.physical_path());
          let description: TextureDescription =
            TextureDescription::describe(&assets.search(&roots)?, request.source.clone(), roots.clone())?;

          TauriResult::Ok(TextureRenderSource::new(&description, &request, assets))
        })
        .await??;

      Some(Arc::new(source))
    }
    None => None,
  };

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
