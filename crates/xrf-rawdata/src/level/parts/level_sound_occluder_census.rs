/// What a sound occluder's decompile could not give back exactly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelSoundOccluderCensus {
  pub polygons: usize,
  /// Occlusion factors no game material of the library has, whose polygons take `default`.
  pub unmatched_factors: Vec<f32>,
}
