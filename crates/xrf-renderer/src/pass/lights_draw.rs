use crate::pass::light_contact::LightContact;

/// How every light is added to the light target this frame.
#[derive(Clone, Copy, Debug)]
pub struct LightsDraw {
  /// Whether the contact shadows' marched shares are read.
  pub contact: LightContact,
  /// Whether the lights' specular is split into the specular target, drawn into second.
  pub is_split: bool,
}
