//! The bump pair a descriptor declares, the inputs it resolves to, and what resolving them came to.

mod xray_bump_fallback;
mod xray_bump_mode;
mod xray_bump_naming;
mod xray_bump_outcome;
mod xray_material_bump;
mod xray_material_bump_input;

pub use xray_bump_fallback::XrayBumpFallback;
pub use xray_bump_mode::XrayBumpMode;
pub use xray_bump_naming::XrayBumpNaming;
pub use xray_bump_outcome::XrayBumpOutcome;
pub use xray_material_bump::XrayMaterialBump;
pub use xray_material_bump_input::XrayMaterialBumpInput;
