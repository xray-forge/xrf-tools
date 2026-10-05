/// Where a value pushed to an upload ring lies this frame: the dynamic offset it is bound at, and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UploadSlice {
  pub offset: u32,
  pub size: u32,
}
