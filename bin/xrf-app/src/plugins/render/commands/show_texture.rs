use std::sync::Arc;

use tauri::State;
use xrf_renderer::{RenderLevelSource, RenderViewportId};
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;
use crate::plugins::textures::description::TextureDescription;
use crate::plugins::textures::surface::render_source::TextureRenderSource;
use crate::plugins::textures::surface::texture_surface_request::TextureSurfaceRequest;

/// Draw a texture laid on a body in a viewport, its files read by the renderer; no request draws none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_texture"))]
#[tauri::command(rename = "show_texture")]
pub async fn render_show_texture(
  state: State<'_, RenderState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  request: Option<TextureSurfaceRequest>,
) -> TauriResult {
  // Taken before the description's hop, so a texture asked for later is never replaced by this one.
  let ticket: u64 = state.ask_show(viewport);
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
          let description: TextureDescription = assets.with_probe(&roots, |probe| {
            TextureDescription::describe(probe, request.source.clone(), roots.clone())
          })??;

          TauriResult::Ok(TextureRenderSource::new(&description, &request, assets))
        })
        .await??;

      Some(Arc::new(source))
    }
    None => None,
  };

  state.show(viewport, ticket, source);

  Ok(())
}
