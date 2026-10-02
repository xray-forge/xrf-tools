/// A viewport's frame as it was presented: eight bit RGBA, rows top to bottom.
#[derive(Clone, Debug)]
pub struct RenderCapture {
  pub width: u32,
  pub height: u32,
  pub pixels: Vec<u8>,
}
