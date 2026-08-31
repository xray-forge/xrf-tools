use uuid::Uuid;

/// Identity of one opening, reissued by every open and every close.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslationSessionId(Uuid);

impl TranslationSessionId {
  pub(super) fn new() -> Self {
    Self(Uuid::new_v4())
  }
}
