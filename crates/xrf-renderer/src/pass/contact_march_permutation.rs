use xrf_renderer_core::ShaderPermutation;

/// Whether the lamps' visibility and light read the contact shadows marched for them (`frame/light_visibility`,
/// `frame/lights`).
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct ContactMarchPermutation {
  pub contact_marched: bool,
}
