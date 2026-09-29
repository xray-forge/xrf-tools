//! The engine's epsilons (`xrCore/math_constants.h`), the limits it compares floats against, under the names it gives
//! them so a port reads against its source.

/// `EPS_S`, the smallest: what `fis_zero` compares against by default.
pub const EPS_S: f32 = 0.000_000_1;

/// `EPS`.
pub const EPS: f32 = 0.000_01;

/// `EPS_L`, the largest.
pub const EPS_L: f32 = 0.001;
