use std::sync::{Arc, MutexGuard};

use xrf_renderer::{RenderLevelSource, RenderSceneId};
use xrf_world::{World, WorldLevelReload};

use crate::core::assets::{AssetChangeEvent, AssetChangeSubscriber, AssetMountState};
use crate::core::session::{SessionChange, SessionRootChange, SessionSnapshot, Sessions};
use crate::plugins::levels::render_source::LevelRenderSource;
use crate::plugins::levels::shown_levels::ShownLevels;
use crate::plugins::levels::state::SelectedLevel;
use crate::plugins::render::world_state::WorldState;

/// Moves the scene of each level shown along with its session through every commit, in the backend: a kept value's
/// scene goes on from the next snapshot, reading again only what its stale parts fed; a value derived again streams
/// afresh; one being derived, left stale or failed stays as it is drawn, since its session kept the snapshot it was
/// drawn from.
pub(crate) struct LevelSceneFollower {
  pub levels: Sessions<SelectedLevel>,
  pub shown: ShownLevels,
  pub world: WorldState,
}

impl LevelSceneFollower {
  /// Moves one session's scene as `change` says, answering the scene it shows now; none where nothing shows the
  /// session, or nothing about it moved.
  fn follow(&self, assets: &AssetMountState, change: &SessionChange) -> Option<RenderSceneId> {
    let shown: Arc<LevelRenderSource> = self.shown.find_source(change.session_id)?;
    let is_derived: bool = match &change.root {
      SessionRootChange::Kept => false,
      SessionRootChange::Derived => true,
      SessionRootChange::Deriving => {
        log::info!(
          "Level '{}' stays as drawn while its session reads it again",
          shown.get_level().source.get_label()
        );

        return None;
      }
      SessionRootChange::Stale | SessionRootChange::Failed { .. } => {
        log::info!(
          "Level '{}' stays as drawn: its session kept revision {}",
          shown.get_level().source.get_label(),
          change.revision.get()
        );

        return None;
      }
    };
    // The snapshot the change left, or a later one where another commit moved the session on since.
    let next: Arc<SessionSnapshot<SelectedLevel>> = self
      .levels
      .require(change.session_id)
      .ok()
      .filter(|next| next.revision >= change.revision)?;
    let key: String = LevelRenderSource::to_scene_key(change.session_id);
    let source: Arc<LevelRenderSource> = match is_derived {
      true => Arc::new(LevelRenderSource::new(next, AssetMountState::clone(assets))),
      false => Arc::new(shown.follow(next)),
    };
    let drawn: Arc<dyn RenderLevelSource> = Arc::clone(&source) as Arc<dyn RenderLevelSource>;
    let reload: WorldLevelReload = LevelRenderSource::to_reload(&change.stale_parts);
    let mut world: MutexGuard<'_, World> = self.world.lock();
    let scene: Option<RenderSceneId> = match is_derived {
      true => world.reopen_scene(&key, drawn),
      false => world.reload_scene(&key, drawn, reload),
    };

    drop(world);

    // Only a scene holds the next source; one nothing took leaves the source drawn before in place.
    if scene.is_some() {
      self.shown.set_source(&source);
    }

    log::info!(
      "Level '{}' follows revision {} in scene {scene:?}: {}",
      source.get_level().source.get_label(),
      change.revision.get(),
      match is_derived {
        true => "streamed afresh".to_owned(),
        false => format!("{reload:?}"),
      }
    );

    scene
  }
}

impl AssetChangeSubscriber for LevelSceneFollower {
  fn take_change(&self, assets: &AssetMountState, event: &AssetChangeEvent) {
    for change in &event.sessions {
      self.follow(assets, change);
    }
  }
}
