use xrf_game_objects::ObjectId;

/// An event about one object, which an `EventBuffer` tells in that object's id order.
pub trait ObjectEvent {
  /// The object it is about.
  fn get_object(&self) -> ObjectId;
}
