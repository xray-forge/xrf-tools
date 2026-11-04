use crate::acoustics::effects::ear_blocks::EarBlocks;

/// One mixer block of a placed voice through its effects: after the direct path, which the listener's reverb is sent,
/// and at each ear.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlacedBlock<'a> {
  pub(crate) direct: &'a [f32],
  pub(crate) ears: EarBlocks<'a>,
}
