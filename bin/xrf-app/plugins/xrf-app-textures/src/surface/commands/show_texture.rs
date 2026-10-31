use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::ViewportShows;
use xrf_renderer::RenderViewportId;
use xrf_world::WorldLevelSource;

use crate::describe::TextureDescription;
use crate::surface::{TextureRenderSource, TexturesShowRequest};

/// Draw a texture laid on a body in a viewport, its files read by the renderer; no request draws none. Answers the
/// scene drawn, which the viewport's load reports name, or none.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "show_texture")]
pub async fn textures_show_texture(
  shows: State<'_, ViewportShows>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  request: Option<TexturesShowRequest>,
) -> TauriResult<Option<u32>> {
  // Taken before the description's hop, so a texture asked for later is never replaced by this one.
  let ticket: u64 = shows.ask(viewport);
  let source: Option<Arc<dyn WorldLevelSource>> = match request {
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
          let description: TextureDescription =
            TextureDescription::describe_in(&assets, request.source.clone(), request.roots.clone())?;

          TauriResult::Ok(TextureRenderSource::new(&description, &request, assets))
        })
        .await??;

      Some(Arc::new(source))
    }
    None => None,
  };

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
