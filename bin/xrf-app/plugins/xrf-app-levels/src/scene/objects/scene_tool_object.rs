use xrf_editable_level::{
  EditableCustomObject, EditableFogVolumeObject, EditableGlowObject, EditableLightObject, EditableParticleObject,
  EditablePortalObject, EditableSceneObject, EditableSectorObject, EditableShapeObject, EditableSoundEnvironmentObject,
  EditableSoundSourceObject, EditableSpawnPointObject, EditableToolObject, EditableWayObject,
};
use xrf_vfs::XraySearch;

use crate::scene::{SceneDrawnObject, SceneObjectProperties};

/// An object tool's typed object as the page is shown it: what every object saves first, and what its kind adds.
pub(crate) trait SceneToolObject: EditableToolObject {
  /// Its flags, name and transform.
  fn get_custom(&self) -> &EditableCustomObject;

  /// What its kind keeps, its library object looked for through `objects` where it places one.
  fn describe(&self, objects: &XraySearch) -> SceneObjectProperties;
}

impl SceneToolObject for EditableSceneObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  /// Its `.object` looked for by its reference below the objects roots; a reference no lookup takes is not found.
  fn describe(&self, objects: &XraySearch) -> SceneObjectProperties {
    SceneObjectProperties::SceneObject {
      reference: self.reference.clone(),
      flags: self.flags,
      is_found: objects
        .find(&SceneDrawnObject::format_object_path(self))
        .is_ok_and(|found| found.is_some()),
    }
  }
}

impl SceneToolObject for EditableSectorObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditablePortalObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableLightObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableGlowObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableSpawnPointObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableShapeObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableWayObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableParticleObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableSoundSourceObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableSoundEnvironmentObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}

impl SceneToolObject for EditableFogVolumeObject {
  fn get_custom(&self) -> &EditableCustomObject {
    &self.object
  }

  fn describe(&self, _: &XraySearch) -> SceneObjectProperties {
    self.into()
  }
}
