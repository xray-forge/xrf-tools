use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_vfs::XrayLogicalPath;

/// A volume set's files by engine identity, built on the first lookup that misses the authored name: a reference
/// written in another case or with other separators then costs one hash lookup rather than a walk of the name table.
#[derive(Clone, Debug, Default)]
pub struct ArchiveNameIndex {
  names: OnceLock<HashMap<String, Arc<str>>>,
}

impl ArchiveNameIndex {
  /// The file `project` lists for an engine path, however the path is spelled; none for a directory or a path it does
  /// not hold.
  pub fn find<'a>(&self, project: &'a ArchiveProject, path: &str) -> Option<&'a ArchiveFileDescriptor> {
    let descriptor: &ArchiveFileDescriptor = match project.files.get(path) {
      Some(descriptor) => descriptor,
      None => {
        let identity: XrayLogicalPath = XrayLogicalPath::new(path).ok()?;
        let name: &Arc<str> = self.names.get_or_init(|| Self::index(project)).get(identity.as_str())?;

        project.files.get(name)?
      }
    };

    (!descriptor.is_directory).then_some(descriptor)
  }

  /// The name `project` lists an engine path's file under, however the path is spelled, or `path` as given where it
  /// lists no file there, so a read of it fails as a read of an absent name does.
  pub fn resolve<'a>(&self, project: &'a ArchiveProject, path: &'a str) -> &'a str {
    self.find(project, path).map_or(path, |descriptor| &descriptor.name)
  }

  fn index(project: &ArchiveProject) -> HashMap<String, Arc<str>> {
    project
      .files
      .iter()
      .filter(|(_, descriptor)| !descriptor.is_directory)
      .filter_map(|(name, _)| Some((XrayLogicalPath::new(name).ok()?.as_str().to_owned(), Arc::clone(name))))
      .collect()
  }
}
