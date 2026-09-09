use std::collections::HashMap;

use xrf_vfs::XrayAssetContainer;

use crate::patch::compare::ArchivePatchOrigin;

/// The distinct origins a comparison met, in the order it met them.
#[derive(Debug, Default)]
pub(crate) struct ArchivePatchOrigins {
  entries: Vec<ArchivePatchOrigin>,
  positions: HashMap<ArchivePatchOrigin, u32>,
}

impl ArchivePatchOrigins {
  /// The index naming `container`, adding it when this is the first entry to come out of it.
  pub(crate) fn intern(&mut self, container: &XrayAssetContainer) -> u32 {
    let origin: ArchivePatchOrigin = ArchivePatchOrigin::from(container);

    if let Some(position) = self.positions.get(&origin) {
      return *position;
    }

    let position: u32 = self.entries.len() as u32;

    self.entries.push(origin.clone());
    self.positions.insert(origin, position);

    position
  }

  /// The table itself, for the report every index is read against.
  pub(crate) fn into_entries(self) -> Vec<ArchivePatchOrigin> {
    self.entries
  }
}
