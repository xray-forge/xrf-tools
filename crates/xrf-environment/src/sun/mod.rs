//! The sun: its sprites and flares, and Monolith's table of where it stands.

pub(crate) mod lens_flare;
pub(crate) mod lens_flare_key;
pub(crate) mod sun_position_key;
pub(crate) mod sun_table;

pub use crate::sun::lens_flare::LensFlare;
pub use crate::sun::lens_flare_key::LensFlareKey;
pub use crate::sun::sun_position_key::SunPositionKey;
pub use crate::sun::sun_table::SunTable;
