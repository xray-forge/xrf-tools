use crate::scene::{SceneObjectId, SceneToolKind};

/// The bits of a scene helper's id that hold its object's index among its tool's objects; the bits above them hold the
/// tool's place in the editor's order of its tools.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_HELPER_INDEX_BITS: u32 = 24;

/// The id a scene object's helper carries, which a pick of the helper answers: its tool's place in
/// `SceneToolKind::ALL` in the top eight bits and its index among the tool's objects in the low
/// [`SCENE_HELPER_INDEX_BITS`], so an id names its object both ways with no listing at hand, and ids sort by tool, then
/// by index. The page reads and writes it the same way (`scene-helper-id.ts`).
///
/// An object past the sixteen million and more an index holds draws no helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SceneHelperId(pub u32);

impl SceneHelperId {
  /// The highest index an id holds.
  pub(crate) const MAX_INDEX: u32 = (1 << SCENE_HELPER_INDEX_BITS) - 1;

  /// The id of `object`'s helper; none for an index past [`Self::MAX_INDEX`].
  pub(crate) fn of(object: &SceneObjectId) -> Option<Self> {
    (object.index <= Self::MAX_INDEX)
      .then(|| Self((object.tool.get_ordinal() << SCENE_HELPER_INDEX_BITS) | object.index))
  }

  /// The scene object the helper of this id stands for; none for an id no tool's place answers. The page reads a
  /// helper's pick back itself (`toSceneHelperObject`), so only the tests read it here, proving both read it alike.
  #[cfg(test)]
  pub(crate) fn get_object(self) -> Option<SceneObjectId> {
    Some(SceneObjectId {
      tool: *SceneToolKind::ALL.get((self.0 >> SCENE_HELPER_INDEX_BITS) as usize)?,
      index: self.0 & Self::MAX_INDEX,
    })
  }
}

// A tool's place takes the bits above the index, so every tool must fit there.
const _: () = assert!(SceneToolKind::ALL.len() <= 1 << (u32::BITS - SCENE_HELPER_INDEX_BITS));
