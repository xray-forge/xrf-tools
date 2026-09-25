use xrf_ogf::OgfFile;

use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::pack::visual::visual_skeleton::convert_bones;

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
  /// The bind pose, or `None` for a visual with no bones, or none it can resolve.
  pub fn of_bind(file: &OgfFile) -> Option<Self> {
    let bones = convert_bones(
      &file.bones.as_ref()?.bones,
      file.ik_data.as_ref().map(|it| it.bones.as_slice()),
    );

    Self::of(
      bones.iter().map(|it| it.name.clone()).collect(),
      bones
        .iter()
        .map(|it| it.bind_transform.clone())
        .collect::<Option<Vec<_>>>()?,
    )
  }

  /// A pose from transforms twelve floats a bone, basis then translation, as a baked motion's frame holds them.
  pub fn of_floats(names: Vec<String>, floats: &[f32]) -> Option<Self> {
    let transforms: Vec<VisualTransform> = floats
      .as_chunks::<12>()
      .0
      .iter()
      .take(names.len())
      .map(|it| VisualTransform {
        i: xrf_math::Vector3d::new(it[0], it[1], it[2]),
        j: xrf_math::Vector3d::new(it[3], it[4], it[5]),
        k: xrf_math::Vector3d::new(it[6], it[7], it[8]),
        c: xrf_math::Vector3d::new(it[9], it[10], it[11]),
      })
      .collect();

    Self::of(names, transforms)
  }

  /// The transform of a bone by name, as the engine finds one: `LL_BoneID`, without regard to case.
  pub fn find(&self, name: &str) -> Option<&VisualTransform> {
    let index: usize = self.names.iter().position(|it| it.eq_ignore_ascii_case(name))?;

    self.transforms.get(index)
  }

  fn of(names: Vec<String>, transforms: Vec<VisualTransform>) -> Option<Self> {
    (names.len() == transforms.len() && !names.is_empty()).then_some(Self { names, transforms })
  }
}
