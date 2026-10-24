use xrf_app_core::session::Sessions;

use crate::PosedVisualMotion;

/// The motions viewers pose their models by, each an opening of the viewer's own rather than a part of the model.
pub type VisualPoses = Sessions<PosedVisualMotion>;
