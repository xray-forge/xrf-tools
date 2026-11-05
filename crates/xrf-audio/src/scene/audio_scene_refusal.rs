/// A sound a scene could not play, and why: it could not be read or decoded, or no voice was left for it.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioSceneRefusal {
  pub sound: String,
  pub reason: String,
}

impl AudioSceneRefusal {
  /// Notes in `refused` that `sound` was refused for `reason`, once a sound however many emitters play it.
  pub(crate) fn record(refused: &mut Vec<Self>, sound: &str, reason: String) {
    if !refused.iter().any(|refusal| *refusal.sound == *sound) {
      refused.push(Self {
        sound: sound.to_owned(),
        reason,
      });
    }
  }
}
