use std::collections::BTreeMap;

use crate::id::object_id::ObjectId;

/// Which ids the registry hands out next. A freed slot is taken again before a new one, the lowest first, so the tables
/// stay dense and the order repeats every run; it comes back a generation on, so an id of its earlier object names
/// nothing. A slot freed at its last generation is retired, never handed out again.
///
/// X-Ray's generator (`CID_Generator`, `xrGame/id_generator.h`) hands out never-used ids before freed ones because its
/// sixteen-bit ids carry no generation, and still runs out of them (`Not enough IDs`); the generation makes reuse safe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ObjectIds {
  /// The first slot never handed out, past the last once every slot has been.
  next: u64,
  /// The freed slots, each with the generation it is handed out at next.
  free: BTreeMap<u32, u32>,
}

impl ObjectIds {
  /// Takes the id the next object created gets, none once every slot is taken or retired.
  pub(crate) fn take(&mut self) -> Option<ObjectId> {
    if let Some((index, generation)) = self.free.pop_first() {
      return ObjectId::new(index, generation);
    }

    let index: u32 = u32::try_from(self.next).ok()?;

    self.next += 1;

    Some(ObjectId::of_index(index))
  }

  /// Marks `id` as taken by an object put there whole, as a new game's or a save's are: creation goes on past it, and
  /// its slot is no longer free.
  pub(crate) fn reserve(&mut self, id: ObjectId) {
    self.next = self.next.max(u64::from(id.get_index()) + 1);
    self.free.remove(&id.get_index());
  }

  /// Gives back `id`, whose object is released: its slot is handed out again at the next generation, or retired.
  pub(crate) fn release(&mut self, id: ObjectId) {
    if let Some(next) = id.get_next_generation() {
      self.free.insert(next.get_index(), next.get_generation());
    }
  }
}
