/// A weather texture as it was last uploaded: its size, a face's for a cube, and the upload's number among every
/// weather texture's, so a file read again under its name, at its size or not, is told apart from what was there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WeatherTextureUpload {
  pub(crate) size: (u32, u32),
  pub(crate) number: u64,
}
