use xrf_editable_object::EditableObjectFile;

use crate::scene::SceneSurfaceKind;

/// A library object's faces parted by the kind of their surfaces (`SceneSurfaceKinds::split`), each part the object
/// with the other kinds' faces left out; none for a kind it holds no face of.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct SceneObjectParts {
  parts: [Option<EditableObjectFile>; SceneSurfaceKind::ALL.len()],
}

impl SceneObjectParts {
  /// `object` whole, every face of it of `kind`.
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
