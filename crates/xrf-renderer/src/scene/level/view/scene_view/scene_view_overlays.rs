use crate::contract::overlay::render_overlay::RenderOverlay;
use crate::contract::overlay::render_selection::RenderSelection;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::page::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::page::helpers::view::helper_sources::HelperSources;
use crate::scene::level::page::level_overlays::LevelOverlays;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::static_scene::draw::static_selection::StaticSelection;

impl SceneView {
  /// Makes what it draws over its frame where the page's overlays or the selection's box changed: the page's, then a
  /// selected spawned object's box.
  pub fn set_overlays(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    overlays: &[RenderOverlay],
    version: u64,
    selection: Option<&RenderSelection>,
  ) {
    self.features.overlays.set(scene, device, overlays, version, selection);
  }

  /// What a selection marks in the scene now, resolved again only once the target or the scene changed; none where
  /// nothing is selected or what it names is not in the scene yet.
  pub fn resolve_selection(
    &mut self,
    scene: &LevelScene,
    selection: Option<&RenderSelection>,
  ) -> Option<&StaticSelection> {
    self.features.overlays.resolve_selection(scene, selection)
  }

  /// What it draws over its frame, once made.
  pub fn get_overlays(&self) -> Option<&LevelOverlays> {
    self.features.overlays.get_overlays()
  }

  /// Builds its helpers' wires and marks again where the scene's helpers, its spawned objects or the options `sources`
  /// hold moved since.
  pub fn set_helpers(&mut self, device: &wgpu::Device, sources: HelperSources<'_>) {
    self.features.helpers.set(device, sources);
  }

  /// The helpers' wires and icons it draws over its frame.
  pub fn get_helper_draws(&self) -> HelperDraws<'_> {
    self.features.helpers.get_draws()
  }
}
