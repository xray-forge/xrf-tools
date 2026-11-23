use crate::feature::lighting::shadows::cascades::cascade_draw::CascadeDraw;
use crate::feature::lighting::shadows::level_shadows::BANDED_FROM;

/// What a cascade keeping a static layer is due by this frame, as read off the cascade and the frame: the whole of the
/// choice of its draw, apart from the maps it draws into.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CascadeSplitState {
  pub index: usize,
  /// What moves every frame lands in it this frame.
  pub has_moving: bool,
  /// What moves every frame was drawn into it last time, which a draw without it must clear.
  pub has_drawn_moving: bool,
  /// It keeps a layer of its trees as last drawn, which what moves every frame is drawn over between their redraws.
  pub is_kept: bool,
  /// Its static layer still holds at the cascade's fit, nothing having changed under it.
  pub is_held: bool,
  /// Its trees must be drawn again whatever their sway: never drawn, its static layer not held or swapped for the one
  /// drawn ahead, or the camera left where their shadows were kept.
  pub is_stale: bool,
  /// The trees' sway shows in it at its texels since it was drawn.
  pub is_swayed: bool,
  /// Its trees may wait another frame for another cascade's redraw, within `TREES_WAIT_MOST` intervals.
  pub can_wait: bool,
  /// The band a sweep of the trees' sway under way draws next.
  pub sweep: Option<u32>,
}

impl CascadeSplitState {
  /// The draw the cascade asks for this frame and whether it is drawn at once whatever its stagger; none where nothing
  /// is due. One what moves every frame lands in draws every frame over its kept layer while its trees wait their
  /// interval, and one whose trees are due for the sway alone waits a frame while another kept cascade redraws its
  /// trees (`is_trees_taken`, which this sets where it redraws its own), so the redraws spread over frames. A sway-only
  /// redraw goes band by band, the near cascades' bands at once so the sweep keeps the sway's rate.
  pub(crate) fn choose(&self, is_trees_taken: &mut bool) -> Option<(CascadeDraw, bool)> {
    let is_kept: bool = self.has_moving && self.is_kept;
    let is_trees_due: bool = self.is_stale || (self.is_swayed && !(is_kept && *is_trees_taken && self.can_wait));

    if is_kept && is_trees_due {
      *is_trees_taken = true;
    }

    if !self.has_moving && !self.is_stale && (self.sweep.is_some() || self.is_swayed) {
      return Some((CascadeDraw::Banded(self.sweep.unwrap_or(0)), self.index < BANDED_FROM));
    }

    if is_kept && !is_trees_due {
      return Some((CascadeDraw::Kept, true));
    }

    let has_moved: bool = self.has_moving || self.has_drawn_moving;

    if !(is_trees_due || has_moved) {
      return None;
    }

    let draw: CascadeDraw = if self.is_held {
      CascadeDraw::Swaying
    } else {
      CascadeDraw::Static
    };

    Some((draw, has_moved))
  }
}
