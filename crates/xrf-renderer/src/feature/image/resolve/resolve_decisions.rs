/// What a view's resolve decided for its frame, prepared before every other unit so each reads this frame's choice
/// rather than the last one's.
#[derive(Clone, Copy, Default)]
pub struct ResolveDecisions {
  /// Frames a temporal resolve gathered, or a reference's sample, which the screen-space effects turn their noise by.
  pub noise_frame: u32,
  /// The reference's sample the frame is drawn as; none outside a reference.
  pub reference_sample: Option<u32>,
  /// Whether DLSS reconstructs rays as it resolves the frame, which the traced and raw signals follow.
  pub is_ray_reconstructed: bool,
}
