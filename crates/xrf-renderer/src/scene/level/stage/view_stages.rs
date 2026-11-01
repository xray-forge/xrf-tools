use crate::feature::image::image_stage::ImageStage;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::stage::effect_stage::EffectStage;
use crate::scene::level::stage::lighting_stage::LightingStage;
use crate::scene::level::stage::scene_stage::SceneStage;

/// A view's stages in the order its frame draws them: the scene, the lighting, the effects over the lit frame and the
/// image it shows.
pub struct ViewStages {
  pub scene: SceneStage,
  pub lighting: LightingStage,
  pub effects: EffectStage,
  pub image: ImageStage,
}

impl ViewStages {
  /// A view's stages, nothing of them drawn yet.
  pub fn new() -> Self {
    Self {
      scene: SceneStage::default(),
      lighting: LightingStage::default(),
      effects: EffectStage::new(),
      image: ImageStage::default(),
    }
  }

  /// Every stage's units, as what each answers alike.
  pub fn list(&self) -> impl Iterator<Item = &dyn ViewFeature> {
    let Self {
      scene,
      lighting,
      effects,
      image,
    } = self;

    scene
      .list()
      .into_iter()
      .chain(lighting.list())
      .chain(effects.list())
      .chain(image.list())
  }

  /// Every stage's units, to let go of or change alike.
  pub fn list_mut(&mut self) -> impl Iterator<Item = &mut dyn ViewFeature> {
    let Self {
      scene,
      lighting,
      effects,
      image,
    } = self;

    scene
      .list_mut()
      .into_iter()
      .chain(lighting.list_mut())
      .chain(effects.list_mut())
      .chain(image.list_mut())
  }
}
