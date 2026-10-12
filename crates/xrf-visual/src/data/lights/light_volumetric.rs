/// What a light's owner sets of its shafts (`IRender_Light::set_volumetric` and its quality, intensity and distance):
/// whether it asks for them, a hanging lamp's spawn flag or a zone's `idle_light_volumetric`; how finely, how bright
/// and how far along its range they are drawn. Monolith draws shafts for every shadowed light whose owner sets them,
/// asked or not, with its own constants (`ps_ssfx_volumetric`); OpenXRay for those asked alone, with these values.
#[derive(Clone, Debug, PartialEq)]
pub struct LightVolumetric {
  pub is_asked: bool,
  pub quality: f32,
  pub intensity: f32,
  pub distance: f32,
}
