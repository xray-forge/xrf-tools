use serde::Serialize;

use crate::level::level_spawn_verbatim_group::LevelSpawnVerbatimGroup;

/// What unpacking a level's spawn list wrote: how many objects it wrote typed, and those it wrote verbatim, grouped by
/// the reason the typed read refused them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnExport {
  pub typed_objects: usize,
  /// In the order each reason was first met.
  pub verbatim_objects: Vec<LevelSpawnVerbatimGroup>,
}

impl LevelSpawnExport {
  /// How many objects were written verbatim, whatever the reason.
  pub fn sum_verbatim_objects(&self) -> usize {
    self.verbatim_objects.iter().map(|group| group.objects).sum()
  }

  /// Counts an object written typed.
  pub(crate) fn count_typed(&mut self) {
    self.typed_objects += 1;
  }

  /// Counts an object written verbatim for `reason`.
  pub(crate) fn count_verbatim(&mut self, reason: &str) {
    match self.verbatim_objects.iter_mut().find(|group| group.reason == reason) {
      Some(group) => group.objects += 1,
      None => self.verbatim_objects.push(LevelSpawnVerbatimGroup {
        reason: String::from(reason),
        objects: 1,
      }),
    }
  }
}
