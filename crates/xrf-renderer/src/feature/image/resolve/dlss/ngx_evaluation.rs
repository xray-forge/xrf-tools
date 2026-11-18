use std::fmt::Display;
use std::sync::{Mutex, PoisonError};

/// The first refusal of a DLSS feature's evaluations since the prepare last looked, recorded while the frame graph
/// runs.
///
/// NGX refusing at run time refuses every frame after, so the prepare hands the frame to the next resolve.
#[derive(Default)]
pub(crate) struct NgxEvaluation {
  refusal: Mutex<Option<String>>,
}

impl NgxEvaluation {
  /// Answers an evaluation's result where NGX took it, and keeps its refusal where it did not, unless one is kept.
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
