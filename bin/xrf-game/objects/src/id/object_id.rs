/// An object's id, X-Ray's `_OBJECT_ID` (`xrServerEntities/alife_space.h`): sixteen bits, every system's key into the
/// registry's tables, kept by saves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(pub u16);

impl ObjectId {
  /// The id X-Ray writes for none, as an object nothing holds names its parent (`0xffff`).
  pub const NONE: u16 = u16::MAX;

  /// The id `raw` names, none for X-Ray's none.
  pub fn of(raw: u16) -> Option<Self> {
    (raw != Self::NONE).then_some(Self(raw))
  }

  /// Its place in a table.
  pub(crate) fn get_index(self) -> usize {
    usize::from(self.0)
  }
}
