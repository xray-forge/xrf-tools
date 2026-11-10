use std::fmt::Display;
use std::sync::{Mutex, PoisonError};

/// What a DLSS feature's evaluations came to, which its pass records into while the frame graph runs and the next
/// prepare reads: the first refusal since the prepare last looked. NGX refusing at run time refuses again every frame
/// it is asked, so the prepare hands the frame to the next resolve rather than asking again.
#[derive(Default)]
pub(crate) struct NgxEvaluation {
  refusal: Mutex<Option<String>>,
}

impl NgxEvaluation {
  /// Keeps what an evaluation came to: answers what it recorded where NGX took it, and keeps the refusal where it did
  /// not, the first one where several frames are refused before the prepare reads them.
  pub(crate) fn keep<T, E: Display>(&self, evaluated: Result<T, E>) -> Option<T> {
    evaluated
      .map_err(|error| {
        self
          .refusal
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .get_or_insert_with(|| error.to_string());
      })
      .ok()
  }

  /// The first refusal since the last call, taken.
  pub(crate) fn take_refusal(&mut self) -> Option<String> {
    self.refusal.get_mut().unwrap_or_else(PoisonError::into_inner).take()
  }
}
