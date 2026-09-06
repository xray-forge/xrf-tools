/// PNG representation produced from the base mip of a DDS image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DdsPng {
  pub width: u32,
  pub height: u32,
  pub bytes: Vec<u8>,
}
