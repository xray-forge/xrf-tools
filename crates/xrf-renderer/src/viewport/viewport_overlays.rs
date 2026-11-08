use crate::contract::overlay::render_overlay::RenderOverlay;
use crate::host::world::render_scene_id::RenderSceneId;

/// What a viewport draws over its frame, and how many sets it has been given, which its level's vertices follow.
#[derive(Default)]
pub(crate) struct ViewportOverlays {
  pub overlays: Vec<RenderOverlay>,
  pub version: u64,
  /// A set given for a scene the viewport does not show yet, drawn from the frame it does: a model's grid sized to it
  /// never stands over the model it replaces while it loads.
  waiting: Option<(RenderSceneId, Vec<RenderOverlay>)>,
}

impl ViewportOverlays {
  /// Takes another set, the latest given winning: at once where it is for no scene or the one `shown`, else once that
  /// scene is shown.
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

  /// Draws the set waiting for `scene`, now shown, if one does.
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
