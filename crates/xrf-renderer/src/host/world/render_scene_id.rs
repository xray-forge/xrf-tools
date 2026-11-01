/// A scene the world streams into the renderer, never named again once it is gone.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RenderSceneId(pub u32);
