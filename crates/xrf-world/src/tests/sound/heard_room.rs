//! The mixer has one reverb: only one scene's room may set it.

use xrf_renderer::RenderSceneId;

use crate::sound::level::heard_room::HeardRoom;
use crate::sound::level::heard_room_candidate::HeardRoomCandidate;

fn candidate(scene: u32, is_listened: bool, is_bursting: bool) -> HeardRoomCandidate {
  HeardRoomCandidate {
    scene: RenderSceneId(scene),
    is_listened,
    is_bursting,
  }
}

// While a viewport records a burst of another level, both are heard: the listened one keeps the room.
#[test]
fn keeps_the_listened_scenes_room_while_another_bursts() {
  assert_eq!(
    HeardRoom::choose([candidate(2, false, true), candidate(1, true, false)]),
    Some(RenderSceneId(1))
  );
}

#[test]
fn takes_a_bursting_scenes_room_with_none_listened() {
  assert_eq!(
    HeardRoom::choose([
      candidate(3, false, true),
      candidate(1, false, false),
      candidate(2, false, true)
    ]),
    Some(RenderSceneId(2)),
    "the lowest numbered, whatever order the scenes are held in"
  );
}

#[test]
fn chooses_no_room_where_no_scene_is_heard() {
  assert_eq!(HeardRoom::choose([candidate(1, false, false)]), None);
  assert_eq!(HeardRoom::choose([]), None);
}
