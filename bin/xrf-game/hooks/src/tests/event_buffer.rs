use xrf_game_objects::ObjectId;

use crate::events::event_buffer::EventBuffer;
use crate::events::object_event::ObjectEvent;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Told {
  object: ObjectId,
  order: u32,
}

impl ObjectEvent for Told {
  fn get_object(&self) -> ObjectId {
    self.object
  }
}

fn told(index: u32, order: u32) -> Told {
  Told {
    object: ObjectId::of_index(index),
    order,
  }
}

#[test]
fn drains_in_id_order_keeping_each_objects_own_order() {
  let mut buffer: EventBuffer<Told> = EventBuffer::default();

  for event in [told(7, 0), told(2, 1), told(7, 2), told(4, 3), told(2, 4)] {
    buffer.push(event);
  }

  assert_eq!(buffer.len(), 5);
  assert_eq!(
    buffer.drain().collect::<Vec<_>>(),
    vec![told(2, 1), told(2, 4), told(4, 3), told(7, 0), told(7, 2)]
  );
  assert!(buffer.is_empty());
}
