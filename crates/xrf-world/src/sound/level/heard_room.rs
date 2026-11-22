use xrf_renderer::RenderSceneId;

use crate::sound::level::heard_room_candidate::HeardRoomCandidate;

/// Which scene's room the mixer's one reverb follows. Two scenes are heard at once only while a viewport draws a burst
/// of a scene other than the one listened to, and the listener stands in only one room.
pub(crate) struct HeardRoom;

impl HeardRoom {
  /// The listened scene, else, with none listened, the lowest numbered of those bursting; none where no scene is heard,
  /// which leaves the reverb silent.
  pub(crate) fn choose(scenes: impl IntoIterator<Item = HeardRoomCandidate>) -> Option<RenderSceneId> {
    let mut bursting: Option<RenderSceneId> = None;

    for candidate in scenes {
      if candidate.is_listened {
        return Some(candidate.scene);
      }

      if candidate.is_bursting {
        bursting = Some(bursting.map_or(candidate.scene, |chosen| chosen.min(candidate.scene)));
      }
    }

    bursting
  }
}
