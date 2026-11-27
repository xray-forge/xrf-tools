use std::vec::Drain;

use crate::events::object_event::ObjectEvent;

/// What a system tells during one step, held until the step ends and then drained in id order: to the event hooks
/// first, then to the loop's queue for presentation. Kept apart from the step's writes, so a listener reads the state
/// the step left, never one half applied; told in id order, so the order a step's own work found them in (a sweep's
/// two passes, a squad's members after it) never shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventBuffer<E> {
  events: Vec<E>,
}

impl<E> Default for EventBuffer<E> {
  fn default() -> Self {
    Self { events: Vec::new() }
  }
}

impl<E: ObjectEvent> EventBuffer<E> {
  pub fn push(&mut self, event: E) {
    self.events.push(event);
  }

  pub fn len(&self) -> usize {
    self.events.len()
  }

  pub fn is_empty(&self) -> bool {
    self.events.is_empty()
  }

  /// Takes every event out in id order, an object's own in the order told, leaving the buffer empty for the next step.
  pub fn drain(&mut self) -> Drain<'_, E> {
    self.events.sort_by_key(ObjectEvent::get_object);
    self.events.drain(..)
  }
}
