/// `PDomainEnum`: the shape of a region particles are generated in or tested against, by the code the file stores.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticleVolumeKind {
  Point,
  Line,
  Triangle,
  Plane,
  Box,
  Sphere,
  Cylinder,
  Cone,
  Blob,
  Disc,
  Rectangle,
  /// A code no shape has, which nothing is inside or generated in.
  Unknown(u32),
}

impl From<u32> for ParticleVolumeKind {
  fn from(code: u32) -> Self {
    match code {
      0 => Self::Point,
      1 => Self::Line,
      2 => Self::Triangle,
      3 => Self::Plane,
      4 => Self::Box,
      5 => Self::Sphere,
      6 => Self::Cylinder,
      7 => Self::Cone,
      8 => Self::Blob,
      9 => Self::Disc,
      10 => Self::Rectangle,
      code => Self::Unknown(code),
    }
  }
}
