/// How the frame's light is written: the scale it stands at (`light_scale` in `common/lighting`), and whether it is the
/// authored levels' summed light (`LightingUniform::summed_light`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightEncoding {
  pub scale: f32,
  pub is_summed: bool,
}
