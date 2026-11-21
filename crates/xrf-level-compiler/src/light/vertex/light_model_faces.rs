use xrf_level::BuildCformRaycastFace;

/// The faces each multi-use model lights itself through, in model space and their models' order, as records for
/// `ShadowMaterials::load` ([`crate::light::vertex::light_models::LightModels::get_model_faces`]).
#[derive(Clone, Debug, PartialEq)]
pub struct LightModelFaces {
  pub faces: Vec<BuildCformRaycastFace>,
  /// Where each model's faces start among them.
  pub starts: Vec<usize>,
}
