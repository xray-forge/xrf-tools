use xrf_app_core::session::{SessionScopes, Sessions};
use xrf_vfs::XrayRoots;

use crate::posed_visual_motion::PosedVisualMotion;
use crate::selected_visual::SelectedVisual;

/// The models open, the motions they are posed by and the roots browsed, each held by the tool that opened it.
pub struct VisualState {
  pub models: Sessions<SelectedVisual>,
  /// The motion each viewer plays its model by, an opening of the viewer's rather than a part of the model.
  pub poses: Sessions<PosedVisualMotion>,
  pub browses: Sessions<XrayRoots>,
}

impl VisualState {
  /// Whatever scope `scopes` releases closes the models, poses and browses it held.
  pub fn new(scopes: &SessionScopes) -> Self {
    Self {
      models: Sessions::new("visual", scopes),
      poses: Sessions::new("visual motion", scopes),
      browses: Sessions::new("visual browse", scopes),
    }
  }
}
