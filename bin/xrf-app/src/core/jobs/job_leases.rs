use std::collections::BTreeMap;

use uuid::Uuid;

use crate::core::jobs::job_resource::JobResource;

/// Atomic claims held by one registration. Action exclusion and output overlap are independent.
struct HeldLeases {
  group: Option<String>,
  resources: Vec<JobResource>,
}

#[derive(Default)]
pub(super) struct JobLeases {
  held: BTreeMap<Uuid, HeldLeases>,
}

impl JobLeases {
  /// Name a conflicting holder without taking any of the requested claims.
  pub(super) fn find_conflict(&self, group: Option<&str>, resources: &[JobResource]) -> Option<(Uuid, String)> {
    for (id, held) in &self.held {
      if let Some(group) = group
        && held.group.as_deref() == Some(group)
      {
        return Some((*id, group.to_owned()));
      }
      for resource in resources {
        if held.resources.iter().any(|other| resource.overlaps(other)) {
          return Some((*id, resource.describe()));
        }
      }
    }
    None
  }

  /// Called under the same registry lock as the conflict and identity checks.
  pub(super) fn take(&mut self, id: Uuid, group: Option<String>, resources: Vec<JobResource>) {
    self.held.insert(id, HeldLeases { group, resources });
  }

  pub(super) fn release(&mut self, id: Uuid) {
    self.held.remove(&id);
  }
}
