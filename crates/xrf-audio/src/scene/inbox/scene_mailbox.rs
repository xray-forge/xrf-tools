use rtrb::{Consumer, Producer};

use crate::scene::audio_scene_frame::AudioSceneFrame;

/// The control thread's end of a scene's [`crate::AudioSceneInbox`]: the frames the host sent, and the ring they go
/// back through to be filled again.
pub(crate) struct SceneMailbox {
  pub(crate) frames: Consumer<Box<AudioSceneFrame>>,
  pub(crate) spent: Producer<Box<AudioSceneFrame>>,
}
