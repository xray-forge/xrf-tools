use crate::scene::level::helpers::gpu::helper_buffer::HelperBuffer;

/// What one part of a view's helpers draws: its wires, two vertices a segment, and its icons, an instance each; none
/// where it draws none.
#[derive(Clone, Copy, Default)]
pub struct HelperPartDraws<'a> {
  pub wires: Option<&'a HelperBuffer>,
  pub icons: Option<&'a HelperBuffer>,
}
