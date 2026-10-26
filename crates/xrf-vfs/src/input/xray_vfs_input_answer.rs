use crate::input::{XrayInput, XrayInputAnswer, XrayInputKind};
use crate::{XrayLogicalPath, XrayLookupScope, XrayVfs};

impl XrayVfs {
  /// What this snapshot answers for an input now: two answers that differ mean what a build read of it changed between.
  /// A listing's answer moves only with a file created or deleted below it, never with one modified, so a size or
  /// content a listing carried is not followed.
  pub fn answer_input(&self, input: &XrayInput) -> XrayInputAnswer {
    let scopes: &[XrayLookupScope] = input.get_key().get_scopes();

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
