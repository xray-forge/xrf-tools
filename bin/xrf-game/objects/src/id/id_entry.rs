use crate::id::object_id::ObjectId;

/// A value of an id table with the id it is kept for, so a lookup by an id of an older generation of the slot finds
/// nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdEntry<T> {
  pub(crate) id: ObjectId,
  pub(crate) value: T,
}
