use crate::data::visual::skeleton::visual_transform::VisualTransform;

/// Where each bone of a visual stands while the visual stands still, in model and renderer space: the pose a spawned
/// object is placed in, its `idle` cycle's first frame where it plays one, its bind pose otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualRestPose {
  /// Each bone's name, in the visual's bone order.
  pub names: Vec<String>,
  /// Each bone's model-space transform, in the same order.
  pub transforms: Vec<VisualTransform>,
}

impl VisualRestPose {
  /// A pose from transforms twelve floats a bone, basis then translation, as a baked motion's frame holds them.
  pub fn of_floats(names: Vec<String>, floats: &[f32]) -> Option<Self> {
    let transforms: Vec<VisualTransform> = floats
      .as_chunks::<{ VisualTransform::FLOATS }>()
      .0
      .iter()
      .take(names.len())
      .map(VisualTransform::from_floats)
      .collect();

    Self::new(names, transforms)
  }

  /// Its transforms as twelve floats a bone, basis then translation, the layout [`Self::of_floats`] reads.
  pub fn to_floats(&self) -> Vec<f32> {
    self.transforms.iter().flat_map(VisualTransform::to_floats).collect()
  }

  /// The transform of a bone by name, as the engine finds one: `LL_BoneID`, without regard to case.
  pub fn find(&self, name: &str) -> Option<&VisualTransform> {
    let index: usize = self.names.iter().position(|it| it.eq_ignore_ascii_case(name))?;

    self.transforms.get(index)
  }

  /// A pose of one transform a bone, or `None` for one naming no bone or naming them unevenly.
  pub(crate) fn new(names: Vec<String>, transforms: Vec<VisualTransform>) -> Option<Self> {
    (names.len() == transforms.len() && !names.is_empty()).then_some(Self { names, transforms })
  }
}
