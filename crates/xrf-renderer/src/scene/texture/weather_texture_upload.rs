/// A weather texture's last upload: its size (one face's for a cube) and a global upload number, which tells a
/// reloaded file apart from the old one even at the same size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WeatherTextureUpload {
  pub(crate) size: (u32, u32),
  pub(crate) number: u64,
}
