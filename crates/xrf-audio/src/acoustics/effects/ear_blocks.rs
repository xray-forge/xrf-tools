/// One mixer block of sound for each ear, as an effect renders it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EarBlocks<'a> {
  pub(crate) left: &'a [f32],
  pub(crate) right: &'a [f32],
}
