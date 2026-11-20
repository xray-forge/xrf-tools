use crate::light::point::light_color::LightColor;
use crate::light::vertex::light_tree_color::LightTreeColor;

/// One placement's light: its vertices', and its tree's scale and bias.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LightReference {
  pub colors: Vec<LightColor>,
  pub scale: LightTreeColor,
  pub bias: LightTreeColor,
}
