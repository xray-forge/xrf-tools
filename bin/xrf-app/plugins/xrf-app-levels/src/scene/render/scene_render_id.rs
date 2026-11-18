use crate::scene::{SceneObjectId, SceneSurfaceKind, SceneToolKind};

/// The bits of a scene render id that hold its object's index among its tool's objects; the bits above them hold the
/// tool's place in the editor's order of its tools.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_RENDER_INDEX_BITS: u32 = 24;

/// The bit a scene render id sets, above every tool's place, to name the model of its object's collision surfaces
/// apart from the object's own: the renderer stands one model a spawn object, so each model needs an id.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_RENDER_COLLISION_BIT: u32 = 1 << (u32::BITS - 1);

/// The bit a scene render id sets, above every tool's place, to name the model of its object's editor-only surfaces
/// apart from the object's own.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_RENDER_EDITOR_BIT: u32 = 1 << (u32::BITS - 2);

/// The bits above every tool's place, each naming one of an object's models apart from its own.
const SCENE_RENDER_PART_BITS: u32 = SCENE_RENDER_COLLISION_BIT | SCENE_RENDER_EDITOR_BIT;

/// Every tool in the editor's order, the place a scene render id's tool bits name each by. Only the page reads it: the
/// backend reads a tool's place off its variant.
#[cfg(feature = "typescript-bindings")]
#[xrf_ipc::constant]
pub(crate) const SCENE_RENDER_TOOLS: &[SceneToolKind] = &SceneToolKind::ALL;

/// The one id a scene object is named to the renderer by, whatever it draws: a model's spawn object, its lighting and
/// its hiding, a selection's target, a helper, and what a pick of any of them answers. Its tool's place in
/// `SceneToolKind::ALL` is in the six bits under the models' two and its index among the tool's objects in the low
/// [`SCENE_RENDER_INDEX_BITS`], so an id names its object both ways with no listing at hand, a spawn point's model and
/// its shape are one object, and ids sort by tool, then by index. A scene object's collision surfaces stand under its
/// id with [`SCENE_RENDER_COLLISION_BIT`] set, its editor-only surfaces with [`SCENE_RENDER_EDITOR_BIT`]. The page
/// reads and writes it the same way (`scene-render-id.ts`).
///
/// An object past the sixteen million and more an index holds is not drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SceneRenderId(pub u32);

impl SceneRenderId {
  /// The highest index an id holds.
  pub(crate) const MAX_INDEX: u32 = (1 << SCENE_RENDER_INDEX_BITS) - 1;

  /// The id `object` is drawn under; none for an index past [`Self::MAX_INDEX`].
  pub(crate) fn of(object: &SceneObjectId) -> Option<Self> {
    (object.index <= Self::MAX_INDEX)
      .then(|| Self((object.tool.get_ordinal() << SCENE_RENDER_INDEX_BITS) | object.index))
  }

  /// The id the model of this object's surfaces of `kind` stands under, the kind's bit set.
  pub(crate) const fn get_part(self, kind: SceneSurfaceKind) -> Self {
    Self(self.0 | kind.get_render_bit())
  }

  /// The scene object this id names, the ids of its other models naming the object too; none for an id no tool's
  /// place answers. The page reads a pick back itself (`toSceneRenderObject`), so only the tests read it here, proving
  /// both read it alike.
  #[cfg(test)]
  pub(crate) fn get_object(self) -> Option<SceneObjectId> {
    let id: u32 = self.0 & !SCENE_RENDER_PART_BITS;

    Some(SceneObjectId {
      tool: *SceneToolKind::ALL.get((id >> SCENE_RENDER_INDEX_BITS) as usize)?,
      index: id & Self::MAX_INDEX,
    })
  }
}

// A tool's place takes the bits between the index and the models' bits, so every tool must fit there.
const SCENE_RENDER_TOOL_BITS: u32 = u32::BITS - SCENE_RENDER_PART_BITS.count_ones() - SCENE_RENDER_INDEX_BITS;
const _: () = assert!(SceneToolKind::ALL.len() <= 1 << SCENE_RENDER_TOOL_BITS);
const _: () = assert!(SCENE_RENDER_PART_BITS.leading_ones() == SCENE_RENDER_PART_BITS.count_ones());
