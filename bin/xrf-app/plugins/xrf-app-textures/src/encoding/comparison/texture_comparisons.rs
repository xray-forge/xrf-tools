use xrf_app_core::session::Sessions;

use crate::encoding::TextureComparison;

/// The comparisons each tool holds, one per texture it weighed.
pub type TextureComparisons = Sessions<TextureComparison>;
