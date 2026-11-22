use xrf_renderer::RenderSceneId;

/// One scene the world plays, as the choice of the room heard sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HeardRoomCandidate {
  pub scene: RenderSceneId,
  /// Whether the listening viewport shows it.
  pub is_listened: bool,
  /// Whether a viewport draws a burst of it, which hears it at the burst's times.
  pub is_bursting: bool,
}
