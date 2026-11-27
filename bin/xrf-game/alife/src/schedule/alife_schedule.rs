use xrf_game_objects::{IdMap, ObjectId, ObjectRegistry, ObjectSchedule};

use crate::schedule::alife_schedule_report::AlifeScheduleReport;

/// Which offline objects A-Life updates, and in what order: X-Ray's schedule registry (`CALifeScheduleRegistry`,
/// `xrGame/alife_schedule_registry.h`, walked by `CSafeMapIterator::update`, `xrGame/safe_map_iterator_inline.h`).
///
/// Kept: each update walks, in id order, the objects of every level that need updating
/// ([`ObjectRegistry::needs_update`]), takes a fixed number of them (`objects_per_update`), and stops early rather than
/// take one twice; the next update takes up after the last one taken. So with many objects each waits its turn, and
/// the cadence the player sees of an offline squad is X-Ray's.
///
/// Not carried over: X-Ray keeps who needs updating as a list beside the objects, added to and taken from as they
/// switch, die or join a squad, which falls out of step with them; here it is read from the registry's tables as the
/// walk reaches each object. X-Ray's `mtALife` runs the whole update on another thread beside the scripts and the
/// client; here it is a step of the loop's own thread, the same every run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlifeSchedule {
  /// Where the next update's walk starts.
  next: ObjectId,
}

impl Default for AlifeSchedule {
  /// A new game's, whose walk starts at the first id as X-Ray's starts at its registry's first object.
  fn default() -> Self {
    Self { next: ObjectId(0) }
  }
}

impl AlifeSchedule {
  /// Where the next update's walk starts.
  pub fn get_next(&self) -> ObjectId {
    self.next
  }

  /// Runs one update at `tick`: takes up to `count` objects that need updating and updates each offline.
  pub fn update(&mut self, objects: &mut ObjectRegistry, count: usize, tick: u64) -> AlifeScheduleReport {
    let due: Vec<ObjectId> = self.take_due(objects, count);
    let schedules: &mut IdMap<ObjectSchedule> = objects.get_schedules_mut();

    // What an object does offline (its graph movement, a squad's or a smart terrain's choices) runs before its stamp, so
    // it reads how long it has waited; nothing does yet.
    for id in &due {
      if let Some(schedule) = schedules.get_mut(*id) {
        schedule.updated = Some(tick);
      }
    }

    AlifeScheduleReport { updated: due.len() }
  }

  /// Takes up to `count` objects that need updating, in id order from where the last update stopped, wrapping round
  /// once at most, and answers them in the order taken.
  pub fn take_due(&mut self, objects: &ObjectRegistry, count: usize) -> Vec<ObjectId> {
    let schedules: &IdMap<ObjectSchedule> = objects.get_schedules();
    let next: ObjectId = self.next;
    let due: Vec<ObjectId> = schedules
      .iter_from(next)
      .chain(schedules.iter().take_while(|(id, _)| *id < next))
      .map(|(id, _)| id)
      .filter(|id| objects.needs_update(*id))
      .take(count)
      .collect();

    // Past the last id the walk starts again at the first, as it wraps round anyway.
    if let Some(last) = due.last() {
      self.next = ObjectId(last.0.wrapping_add(1));
    }

    due
  }
}
