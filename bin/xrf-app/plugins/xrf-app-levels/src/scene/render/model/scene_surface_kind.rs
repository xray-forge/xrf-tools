use xrf_renderer::RenderSpawnCategory;

use crate::scene::render::scene_render_id::{SCENE_RENDER_COLLISION_BIT, SCENE_RENDER_EDITOR_BIT};

/// How a scene draws a library object's surface. Each kind is a model of its own, named after the object's reference,
/// under the object's id with the kind's bit set and in its own group, so the page switches each kind by group alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SceneSurfaceKind {
  /// A surface the game draws, the object's own model.
  Drawn,
  /// A surface whose compiler shader leaves it out of the level's visuals (`Shader_xrLC::bRendering` clear), kept for
  /// collision, as a tree's hull is, or for shadows alone.
  Collision,
  /// A surface the game may draw but a player never sees, dressed in one of the SDK's marker textures
  /// (`prop\prop_fake_*`) to show the editor what it is: a sun blocker, an occluder's sheet, a ladder or a collision
  /// hull.
  EditorOnly,
}

impl SceneSurfaceKind {
  /// Every kind, the object's own model first.
  pub(crate) const ALL: [Self; 3] = [Self::Drawn, Self::Collision, Self::EditorOnly];

  /// The kind of the model `name`, by its suffix; the object's own model where it has none.
  pub(crate) fn of_model(name: &str) -> Self {
    Self::ALL
      .into_iter()
      .find(|kind| !kind.get_suffix().is_empty() && name.ends_with(kind.get_suffix()))
      .unwrap_or(Self::Drawn)
  }

  /// The model name of this kind's surfaces of the library object `reference`.
  pub(crate) fn format_model_name(self, reference: &str) -> String {
    format!("{reference}{}", self.get_suffix())
  }

  /// The library object reference of this kind's model `name`.
  pub(crate) fn get_reference(self, name: &str) -> &str {
    name.strip_suffix(self.get_suffix()).unwrap_or(name)
  }

  /// The bit its model's scene render id sets over the object's.
  pub(crate) const fn get_render_bit(self) -> u32 {
    match self {
      Self::Drawn => 0,
      Self::Collision => SCENE_RENDER_COLLISION_BIT,
      Self::EditorOnly => SCENE_RENDER_EDITOR_BIT,
    }
  }

  /// The group of its models: props, which the page never hides, for the object's own model; for the other kinds the
  /// renderer's weapons and lamps groups, which nothing else of a scene uses.
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

  /// The suffix its model's name adds to the reference, which no reference contains; empty for the object's own model.
  const fn get_suffix(self) -> &'static str {
    match self {
      Self::Drawn => "",
      Self::Collision => "#collision",
      Self::EditorOnly => "#editor",
    }
  }
}
