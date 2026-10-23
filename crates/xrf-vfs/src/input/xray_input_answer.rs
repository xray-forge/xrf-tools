use crate::input::{XrayInput, XrayInputKind, XrayRevision};
use crate::{XrayLogicalPath, XrayLookupScope, XrayMountId, XrayVfs};

/// What a world answers for an input: the source that holds it and its revision there.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct XrayInputAnswer {
  pub source: Option<XrayMountId>,
  pub revision: XrayRevision,
}

impl XrayVfs {
  /// What this world answers for an input now: two answers that differ mean what a build read of it changed between.
  pub fn answer_input(&self, input: &XrayInput) -> XrayInputAnswer {
    let scopes: &[XrayLookupScope] = input.get_search().get_scopes();

    match input.get_kind() {
      XrayInputKind::File(logical_path) => scopes
        .iter()
        .find_map(|scope| self.get_winner_in_scope(scope, logical_path.as_str()))
        .map(|(mount, _)| XrayInputAnswer {
          source: Some(mount.get_id()),
          revision: self.get_revisions().get_file(mount.get_id(), logical_path.as_str()),
        })
        .unwrap_or_default(),
      XrayInputKind::Listing(directory) => {
        let directory: &str = directory.as_ref().map_or("", XrayLogicalPath::as_str);

        XrayInputAnswer {
          source: None,
          revision: scopes
            .iter()
            .flat_map(|scope| self.mounts_in(scope))
            .map(|mount| self.get_revisions().get_directory(mount.get_id(), directory))
            .max()
            .unwrap_or_default(),
        }
      }
    }
  }
}
