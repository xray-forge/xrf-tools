use std::sync::Arc;

/// A sound by its source's scope and its reference.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SoundKey {
  pub(crate) scope: Arc<str>,
  pub(crate) reference: Arc<str>,
}
