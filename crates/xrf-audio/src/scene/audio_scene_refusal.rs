/// A sound a scene could not play, and why: it could not be read or decoded, or no voice was left for it.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioSceneRefusal {
  pub sound: String,
  pub reason: String,
}
