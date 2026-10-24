use xrf_app_core::session::Sessions;

use crate::SelectedVisual;

/// The models open, each held by the tool that opened it.
pub type VisualModels = Sessions<SelectedVisual>;
