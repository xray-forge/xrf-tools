//! The types crossing IPC to the page: the renderer's settings and GPU, a view's options and each feature's settings
//! (`feature/`, grouped as the renderer's features are), the reports, the captures, the overlays and selection, and
//! the viewports with the page's backdrop. Beside them, the window layer a native host draws its own interface with
//! (`layer/`), which no page sends and so is not mirrored to it.

pub(crate) mod capture;
pub(crate) mod feature;
pub(crate) mod layer;
pub(crate) mod overlay;
pub(crate) mod render_color;
pub(crate) mod report;
pub(crate) mod settings;
pub(crate) mod view;
pub(crate) mod viewport;
