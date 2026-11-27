/// A picture the developer layer shows, as decoded: its size in pixels and its pixels, RGBA, row by row from the top.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeveloperPicture {
  pub width: u32,
  pub height: u32,
  pub rgba: Vec<u8>,
}
