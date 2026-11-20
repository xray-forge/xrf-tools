use xrf_editable_object::EditableObjectFile;

use crate::scene::SceneSurfaceKind;

/// A library object split by surface kind (`SceneSurfaceKinds::split`), each part the object without the other kinds'
/// faces; none for a kind it has no face of.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct SceneObjectParts {
  parts: [Option<EditableObjectFile>; SceneSurfaceKind::ALL.len()],
}

impl SceneObjectParts {
  /// `object` whole as the part of `kind`.
  pub(crate) fn whole(kind: SceneSurfaceKind, object: EditableObjectFile) -> Self {
    let mut parts: Self = Self::default();

    parts.insert(kind, object);
    parts
  }

  /// Keeps `part` as the part of `kind`.
  pub(crate) fn insert(&mut self, kind: SceneSurfaceKind, part: EditableObjectFile) {
    self.parts[kind.get_index()] = Some(part);
  }

  /// Takes the part of `kind`; none for a kind it holds no face of, or taken.
  pub(crate) fn take(&mut self, kind: SceneSurfaceKind) -> Option<EditableObjectFile> {
    self.parts[kind.get_index()].take()
  }
}
