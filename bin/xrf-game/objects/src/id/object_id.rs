use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

/// An object's id: a slot of the registry's tables and the generation of the object in it, every table's key. Opaque
/// and a plain number to whatever holds it: 53 bits of a `u64`, the slot index in the low 32 and the generation in the
/// 21 above, so it crosses exactly into a Lua number (a double) and into JSON. The generation lets a freed slot take
/// another object without an id kept by a hook, a save or a memory naming the newcomer: the registry hands the slot out
/// again a generation later, and retires one whose generation runs out.
///
/// Ids order by slot, so a table iterates in slot order whatever generations its objects are of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId(u64);

impl ObjectId {
  /// The bits the slot index takes, the lowest.
  const INDEX_BITS: u32 = 32;

  /// The bits the generation takes, above the index.
  const GENERATION_BITS: u32 = 21;

  /// The last generation a slot reaches; a slot freed at it is retired.
  pub const MAX_GENERATION: u32 = (1 << Self::GENERATION_BITS) - 1;

  /// The id of slot `index` at `generation`, none past the last generation.
  pub fn new(index: u32, generation: u32) -> Option<Self> {
    if generation > Self::MAX_GENERATION {
      return None;
    }

    Some(Self((u64::from(generation) << Self::INDEX_BITS) | u64::from(index)))
  }

  /// The id of the first object slot `index` holds, its generation 0, as a new game's objects take theirs.
  pub fn of_index(index: u32) -> Self {
    Self(u64::from(index))
  }

  /// The id `raw` names, as a script, a save or JSON holds it; none for a number past the 53 bits.
  pub const fn of_raw(raw: u64) -> Option<Self> {
    if raw >> (Self::INDEX_BITS + Self::GENERATION_BITS) != 0 {
      return None;
    }

    Some(Self(raw))
  }

  /// The id as a plain number, as a script, a save or JSON holds it.
  pub const fn get_raw(self) -> u64 {
    self.0
  }

  /// The slot it names.
  pub const fn get_index(self) -> u32 {
    self.0 as u32
  }

  /// Which of the objects its slot has held it is, from 0.
  pub const fn get_generation(self) -> u32 {
    (self.0 >> Self::INDEX_BITS) as u32
  }

  /// The id of the slot's next object, none once its generations are spent.
  pub(crate) fn get_next_generation(self) -> Option<Self> {
    Self::new(self.get_index(), self.get_generation() + 1)
  }

  /// Its slot as a position in a table.
  pub(crate) fn get_slot(self) -> usize {
    usize::try_from(self.get_index()).unwrap_or(usize::MAX)
  }
}

impl Ord for ObjectId {
  /// By slot, then generation.
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .get_index()
      .cmp(&other.get_index())
      .then_with(|| self.get_generation().cmp(&other.get_generation()))
  }
}

impl PartialOrd for ObjectId {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Display for ObjectId {
  /// The plain number a script or a log reads.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    write!(formatter, "{}", self.0)
  }
}
