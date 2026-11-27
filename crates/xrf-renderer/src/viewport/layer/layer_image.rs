/// A layer texture's pixels as its host has set them, kept on the CPU: the texture is made from them on the GPU, and made
/// again from them on a GPU started after one was lost, since a host sends each pixel once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LayerImage {
  pub width: u32,
  pub height: u32,
  /// Eight-bit premultiplied RGBA, display encoded, row after row.
  pub pixels: Vec<u8>,
  pub is_nearest: bool,
}
