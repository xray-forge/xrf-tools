/// A base texture coordinate as a static vertex stores it ([`crate::LevelVertexQuantizer::get_base_coordinate`]): the
/// whole part in a short of the base coordinate element, the fraction in the tangent's or binormal's fourth byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelVertexBaseCoordinate {
  pub primary: i16,
  pub fraction: u8,
}
