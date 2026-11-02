//! The light and weather model on the CPU, which the frame's uniforms are made from: the lamps, the sun and its
//! cascades, the units light is measured in, the weather's values and the wind. The passes lighting the frame are in
//! `feature::lighting`.

pub(crate) mod lamp;
pub(crate) mod luminance;
pub(crate) mod material_slice;
pub(crate) mod srgb;
pub(crate) mod sun;
pub(crate) mod unit;
pub(crate) mod weather;
pub(crate) mod wind;
