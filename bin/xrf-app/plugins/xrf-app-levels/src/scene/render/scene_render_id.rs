use crate::scene::{SceneObjectId, SceneSurfaceKind, SceneToolKind};

/// The low bits of a scene render id, holding its object's index among its tool's objects; the bits above hold the
/// tool's place in the editor's tool order.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_RENDER_INDEX_BITS: u32 = 24;

/// The scene render id bit, above the tool bits, naming the model of an object's collision surfaces apart from its
/// own: the renderer holds one model per spawn object, so each model needs an id.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_RENDER_COLLISION_BIT: u32 = 1 << (u32::BITS - 1);

/// The scene render id bit, above the tool bits, naming the model of an object's editor-only surfaces apart from its
/// own.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_RENDER_EDITOR_BIT: u32 = 1 << (u32::BITS - 2);

/// The bits above every tool's place, each naming one of an object's models apart from its own.
const SCENE_RENDER_PART_BITS: u32 = SCENE_RENDER_COLLISION_BIT | SCENE_RENDER_EDITOR_BIT;

/// Every tool in the editor's order, which a scene render id's tool bits index. Only the page reads it; the backend
/// takes a tool's place from its variant.
#[cfg(feature = "typescript-bindings")]
#[xrf_ipc::constant]
pub(crate) const SCENE_RENDER_TOOLS: &[SceneToolKind] = &SceneToolKind::ALL;

/// The one id a scene object is named to the renderer by, whatever it draws: a model's spawn object, its lighting and
/// hiding, a selection target, a helper, and what a pick of any of them answers. The low [`SCENE_RENDER_INDEX_BITS`]
/// hold its index among its tool's objects, the six bits above its tool's place in `SceneToolKind::ALL`, and the top
/// two mark its collision ([`SCENE_RENDER_COLLISION_BIT`]) and editor-only ([`SCENE_RENDER_EDITOR_BIT`]) surface
/// models. So an id names its object both ways without a listing, a spawn point's model and its shape are one object,
/// and ids sort by tool, then index. The page reads and writes it the same way (`scene-render-id.ts`).
///
/// An object past [`Self::MAX_INDEX`] is not drawn.
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

  /// The id of the model of this object's `kind` surfaces: this id with the kind's bit set.
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
