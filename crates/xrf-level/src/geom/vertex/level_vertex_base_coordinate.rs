/// A base texture coordinate as a static vertex stores it: its whole part in a short of the base coordinate element,
/// and the byte below it in the tangent's or binormal's fourth byte (see
/// [`crate::LevelVertexQuantizer::get_base_coordinate`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelVertexBaseCoordinate {
  pub primary: i16,
  pub fraction: u8,
}
