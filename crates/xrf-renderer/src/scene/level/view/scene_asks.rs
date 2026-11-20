use crate::scene::level::lights::asks::light_asks::LightAsks;

/// What a view asks of the scene it shows this frame, once it has readied its frame from it (`SceneView::prepare`): the
/// lights it sees and keeps ready. The scene settles every view's asks at once (`LevelScene::settle`), then each view
/// finishes its frame from its own (`SceneView::finish_prepare`).
pub(crate) struct SceneAsks {
  pub lights: LightAsks,
}
