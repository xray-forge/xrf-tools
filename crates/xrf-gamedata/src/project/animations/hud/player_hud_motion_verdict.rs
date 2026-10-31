use std::collections::BTreeSet;

/// What judging the requested motions found: a message per motion some hands cannot play, and those hands.
#[derive(Debug, Default)]
pub(crate) struct PlayerHudMotionVerdict {
  pub(crate) messages: Vec<String>,
  pub(crate) invalid_hands: BTreeSet<String>,
}
