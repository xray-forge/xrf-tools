use crate::contract::overlay::render_overlay::RenderOverlay;
use crate::host::world::render_scene_id::RenderSceneId;

/// What a viewport draws over its frame, and how many sets it has been given, which its level's vertices follow.
#[derive(Default)]
pub(crate) struct ViewportOverlays {
  pub overlays: Vec<RenderOverlay>,
  pub version: u64,
  /// A set for a scene not shown yet, drawn once it is, so a grid sized to a new model never covers the old one.
  waiting: Option<(RenderSceneId, Vec<RenderOverlay>)>,
}

impl ViewportOverlays {
  /// Takes a set, replacing any waiting one: at once for no scene or the `shown` one, else once its scene is shown.
  pub(crate) fn set(
    &mut self,
    overlays: Vec<RenderOverlay>,
    scene: Option<RenderSceneId>,
    shown: Option<RenderSceneId>,
  ) {
    self.waiting = None;

    match scene {
      Some(scene) if Some(scene) != shown => self.waiting = Some((scene, overlays)),
      _ => self.replace(overlays),
    }
  }

  /// Applies the set waiting for `scene`, now shown, if any.
  pub(crate) fn show(&mut self, scene: RenderSceneId) {
    if let Some((_, overlays)) = self.waiting.take_if(|(waiting, _)| *waiting == scene) {
      self.replace(overlays);
    }
  }

  /// Takes another set in place of the one it held, a version on.
  fn replace(&mut self, overlays: Vec<RenderOverlay>) {
    self.overlays = overlays;
    self.version += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::ViewportOverlays;
  use crate::contract::overlay::render_overlay::RenderOverlay;
  use crate::host::world::render_scene_id::RenderSceneId;

  const SHOWN: RenderSceneId = RenderSceneId(1);
  const NEXT: RenderSceneId = RenderSceneId(2);

  fn given(count: usize) -> Vec<RenderOverlay> {
    vec![
      RenderOverlay::Skeleton {
        color: [1.0; 3],
        is_depth_tested: false
      };
      count
    ]
  }

  #[test]
  fn draws_a_set_for_no_scene_or_the_one_shown_at_once() {
    let mut overlays: ViewportOverlays = ViewportOverlays::default();

    overlays.set(given(1), None, Some(SHOWN));
    assert_eq!((overlays.overlays.len(), overlays.version), (1, 1));

    overlays.set(given(2), Some(SHOWN), Some(SHOWN));
    assert_eq!((overlays.overlays.len(), overlays.version), (2, 2));
  }

  #[test]
  fn keeps_the_set_drawn_until_the_scene_another_is_for_is_shown() {
    let mut overlays: ViewportOverlays = ViewportOverlays::default();

    overlays.set(given(1), Some(SHOWN), Some(SHOWN));
    overlays.set(given(2), Some(NEXT), Some(SHOWN));
    assert_eq!((overlays.overlays.len(), overlays.version), (1, 1));

    overlays.show(SHOWN);
    assert_eq!(overlays.version, 1);

    overlays.show(NEXT);
    assert_eq!((overlays.overlays.len(), overlays.version), (2, 2));

    overlays.show(NEXT);
    assert_eq!(overlays.version, 2);
  }

  #[test]
  fn lets_a_later_set_replace_one_waiting() {
    let mut overlays: ViewportOverlays = ViewportOverlays::default();

    overlays.set(given(2), Some(NEXT), Some(SHOWN));
    overlays.set(given(3), None, Some(SHOWN));
    overlays.show(NEXT);
    assert_eq!((overlays.overlays.len(), overlays.version), (3, 1));
  }
}
