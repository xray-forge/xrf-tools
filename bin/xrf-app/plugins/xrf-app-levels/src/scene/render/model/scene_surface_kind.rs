use xrf_renderer::RenderSpawnCategory;

use crate::scene::render::scene_render_id::{SCENE_RENDER_COLLISION_BIT, SCENE_RENDER_EDITOR_BIT};

/// How a scene draws a surface of a library object: each kind is a model of its own, named after the object's
/// reference, standing under the object's id with the kind's bit set and in a group of its own, so the page shows each
/// kind all or none by its group alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SceneSurfaceKind {
  /// A surface the game draws, the object's own model.
  Drawn,
  /// A surface whose compiler shader leaves it out of the level's visuals (`Shader_xrLC::bRendering` clear), kept for
  /// collision, as a tree's hull is, or for shadows alone.
  Collision,
  /// A surface the game may draw but a player never sees, dressed in one of the SDK's marker textures
  /// (`prop\prop_fake_*`) to show the editor what it stands for: a sun blocker, an occluder's sheet, a ladder or a
  /// collision hull.
  EditorOnly,
}

impl SceneSurfaceKind {
  /// Every kind, the object's own model first.
  pub(crate) const ALL: [Self; 3] = [Self::Drawn, Self::Collision, Self::EditorOnly];

  /// The kind of the model `name` names: a kind's suffix after a reference, else the object's own model.
  pub(crate) fn of_model(name: &str) -> Self {
    Self::ALL
      .into_iter()
      .find(|kind| !kind.get_suffix().is_empty() && name.ends_with(kind.get_suffix()))
      .unwrap_or(Self::Drawn)
  }

  /// The model the surfaces of this kind of the library object `reference` names draw as.
  pub(crate) fn format_model_name(self, reference: &str) -> String {
    format!("{reference}{}", self.get_suffix())
  }

  /// The reference of the library object whose model of this kind `name` names.
  pub(crate) fn get_reference(self, name: &str) -> &str {
    name.strip_suffix(self.get_suffix()).unwrap_or(name)
  }

  /// The bit its model's scene render id sets over its object's.
  pub(crate) const fn get_render_bit(self) -> u32 {
    match self {
      Self::Drawn => 0,
      Self::Collision => SCENE_RENDER_COLLISION_BIT,
      Self::EditorOnly => SCENE_RENDER_EDITOR_BIT,
    }
  }

  /// The group its models stand in: the props group, which the page never hides, for the object's own model, and for
  /// each other kind a group nothing else of a scene stands in, the renderer's weapons and lamps groups.
  pub(crate) const fn get_category(self) -> RenderSpawnCategory {
    match self {
      Self::Drawn => RenderSpawnCategory::Props,
      Self::Collision => RenderSpawnCategory::Weapons,
      Self::EditorOnly => RenderSpawnCategory::Lamps,
    }
  }

  /// Its place in [`Self::ALL`].
  pub(crate) const fn get_index(self) -> usize {
    self as usize
  }

  /// What its model's name adds after the reference, which no reference spells; nothing for the object's own model.
  const fn get_suffix(self) -> &'static str {
    match self {
      Self::Drawn => "",
      Self::Collision => "#collision",
      Self::EditorOnly => "#editor",
    }
  }
}
