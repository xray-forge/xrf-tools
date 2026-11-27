/// A texture the layer's meshes sample, named by the host that made it; its producer's own ids (egui's) map onto it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderLayerTextureId(pub u64);
