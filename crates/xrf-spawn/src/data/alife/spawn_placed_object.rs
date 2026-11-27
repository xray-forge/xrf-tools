use crate::data::alife::alife_object::AlifeObject;

/// An object of a spawn and the level it stands on.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnPlacedObject {
  pub object: AlifeObject,
  /// The id of the graph level its game vertex stands on (`GraphLevel::id`), none for an object with no game vertex or
  /// one the graph does not hold.
  pub level: Option<u8>,
}
