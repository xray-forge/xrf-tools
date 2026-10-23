use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::XrayVfs;
use crate::input::{XrayInput, XrayInputAnswer};

/// The inputs some reads resolved, each with what its snapshot answered for it then.
#[derive(Clone, Debug, Default)]
pub struct XrayInputRecord {
  inputs: Arc<Mutex<HashMap<XrayInput, XrayInputAnswer>>>,
}

impl XrayInputRecord {
  pub fn new() -> Self {
    Self::default()
  }

  /// Records an input as `vfs` answers it now, unless it is recorded already.
  pub(crate) fn record(&self, vfs: &XrayVfs, input: XrayInput) {
    if self.lock().contains_key(&input) {
      return;
    }

    // Answered outside the lock, so readers recording side by side wait on each other only to file what they found.
    let answer: XrayInputAnswer = vfs.answer_input(&input);

    self.lock().entry(input).or_insert(answer);
  }

  /// Adds what another record holds, keeping the answers this one has already.
  pub fn extend_from(&self, other: &XrayInputRecord) {
    if Arc::ptr_eq(&self.inputs, &other.inputs) {
      return;
    }

    // Copied out first, so the two locks are never held together whichever way round two records join.
    let other: Vec<(XrayInput, XrayInputAnswer)> = other.list();
    let mut inputs: MutexGuard<HashMap<XrayInput, XrayInputAnswer>> = self.lock();

    for (input, answer) in other {
      inputs.entry(input).or_insert(answer);
    }
  }

  /// What was recorded for an input, or `None` where nothing read it.
  pub fn get(&self, input: &XrayInput) -> Option<XrayInputAnswer> {
    self.lock().get(input).copied()
  }

  /// Every input recorded, with its answer, in no particular order.
  pub fn list(&self) -> Vec<(XrayInput, XrayInputAnswer)> {
    self
      .lock()
      .iter()
      .map(|(input, answer)| (input.clone(), *answer))
      .collect()
  }

  pub fn len(&self) -> usize {
    self.lock().len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// The inputs, whatever a panicking reader left: an input is only ever filed whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<XrayInput, XrayInputAnswer>> {
    self.inputs.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
