use xrf_app_core::session::Sessions;

use crate::browse::TextureBrowse;

/// The listings each tool's tree holds open.
pub type TextureBrowses = Sessions<TextureBrowse>;
