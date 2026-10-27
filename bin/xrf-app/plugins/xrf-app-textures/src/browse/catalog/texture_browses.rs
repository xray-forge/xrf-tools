use xrf_app_core::session::Sessions;

use crate::browse::TextureCatalog;

/// The listing each tool's tree holds open, restored whole after a frontend reload.
pub type TextureBrowses = Sessions<TextureCatalog>;
