/// A vertex's position bits: the export copies one source point into every vertex standing at it, so equal bits pair a
/// compiled vertex with a reference one and never merge two that are merely close.
pub(crate) type PositionKey = [u32; 3];
