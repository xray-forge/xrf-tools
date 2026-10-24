use xrf_app_core::session::Sessions;
use xrf_vfs::XrayRoots;

/// The roots each model browser lists, held apart from any model it opens.
pub type VisualBrowses = Sessions<XrayRoots>;
