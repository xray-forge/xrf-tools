//! How a colour stored for the display becomes linear light, and how bright linear light is: the renderer lights with
//! these, and the tools that measure textures measure them as the renderer lights them, so both read them here.

/// The power of the sRGB display's transfer: the standard's reference display, which `xrf-renderer`'s
/// `common/srgb.wgsl` explains.
pub const SRGB_DISPLAY_POWER: f32 = 2.2;

/// Rec. 709's weights of linear red, green and blue in luminance.
pub const LUMINANCE_WEIGHTS: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// Linear light encoded by the sRGB display's transfer, as a colour texture holds it.
pub fn srgb_encoded(linear: f32) -> f32 {
  linear.max(0.0).powf(1.0 / SRGB_DISPLAY_POWER)
}

/// A value encoded by the sRGB display's transfer back to linear light; past one, as an engine light brighter than
/// white is, the power continues.
pub fn srgb_decoded(encoded: f32) -> f32 {
  encoded.max(0.0).powf(SRGB_DISPLAY_POWER)
}

/// A linear colour's luminance, by Rec. 709's weights.
pub fn luminance(colour: [f32; 3]) -> f32 {
  colour
    .iter()
    .zip(LUMINANCE_WEIGHTS)
    .map(|(channel, weight)| channel * weight)
    .sum()
}
