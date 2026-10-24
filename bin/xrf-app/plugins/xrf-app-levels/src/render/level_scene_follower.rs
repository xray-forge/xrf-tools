use std::sync::{Arc, MutexGuard};

use xrf_app_core::assets::{AssetChangeEvent, AssetChangeSubscriber, AssetMountState};
use xrf_app_core::session::{SessionChange, SessionRootChange, SessionSnapshot};
use xrf_app_render::WorldState;
use xrf_renderer::{RenderLevelSource, RenderSceneId};
use xrf_world::{World, WorldLevelReload};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::level_report::LevelReport;
use crate::render::{LevelRenderSource, ShownLevels};

/// Moves the scene of each level shown along with its session through every commit, in the backend: a kept value's
/// scene goes on from the next snapshot, reading again only what its stale parts fed; a value derived again streams
/// afresh; one being derived, left stale or failed stays as it is drawn, since its session kept the snapshot it was
/// drawn from.
pub(crate) struct LevelSceneFollower {
  pub levels: SelectedLevels,
  pub shown: ShownLevels,
  pub world: WorldState,
}

impl LevelSceneFollower {
  /// Moves one session's scene as `change` says; nothing where nothing shows the session, or nothing about it moved.
  fn follow(&self, assets: &AssetMountState, change: &SessionChange) {
    let Some(shown) = self.shown.find_source(change.session_id) else {
      return;
    };
    // A kept level whose geometry or surfaces moved streams afresh as a derived one does: its sectors are packed anew.
    let is_afresh: bool = match &change.root {
      SessionRootChange::Kept | SessionRootChange::Derived => change.is_view_renewed,
      SessionRootChange::Deriving => {
        LevelReport::follow_deriving(&shown.get_level().source);

        return;
      }
      SessionRootChange::Stale | SessionRootChange::Failed { .. } => {
        LevelReport::follow_kept(&shown.get_level().source, change.revision.get());

        return;
      }
    };
    // The snapshot the change left, or a later one where another commit moved the session on since.
    let Some(next) = self
      .levels
      .require(change.session_id)
      .ok()
      .filter(|next: &Arc<SessionSnapshot<SelectedLevel>>| next.revision >= change.revision)
    else {
      return;
    };
    let key: String = LevelRenderSource::to_scene_key(change.session_id);
    let source: Arc<LevelRenderSource> = match is_afresh {
      true => Arc::new(LevelRenderSource::new(next, AssetMountState::clone(assets))),
      false => Arc::new(shown.follow(next)),
    };
    let drawn: Arc<dyn RenderLevelSource> = Arc::clone(&source) as Arc<dyn RenderLevelSource>;
    let reload: WorldLevelReload = LevelRenderSource::to_reload(&change.stale_parts);
    let mut world: MutexGuard<'_, World> = self.world.lock();
    let scene: Option<RenderSceneId> = match is_afresh {
      true => world.reopen_scene(&key, drawn),
      false => world.reload_scene(&key, drawn, reload),
    };

    drop(world);

    // Only a scene holds the next source; one nothing took leaves the source drawn before in place.
    if scene.is_some() {
      self.shown.set_source(&source);
    }

    LevelReport::followed(
      &source.get_level().source,
      change.revision.get(),
      scene,
      (!is_afresh).then_some(&reload),
    );
  }
}

impl AssetChangeSubscriber for LevelSceneFollower {
  fn take_change(&self, assets: &AssetMountState, event: &AssetChangeEvent) {
    for change in &event.sessions {
      self.follow(assets, change);
    }
  }
}
