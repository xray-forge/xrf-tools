use xrf_ogf::{OgfFile, OgfIkDataChunk};

use crate::data::visual::skeleton::visual_rest_pose::VisualRestPose;
use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::pack::tests::fixtures::{MODEL_TYPE_SKELETON_ANIM, bind, bones, vector, visual};
use crate::pack::visual::visual_skeleton::VisualSkeleton;

/// Two bones, the second a metre above the first.
fn two_bones(binds: usize) -> OgfFile {
  OgfFile {
    bones: Some(bones(&[("root", ""), ("Bone_Lamp", "root")])),
    ik_data: Some(OgfIkDataChunk {
      bones: [vector(0.0, 0.0, 0.0), vector(0.0, 1.0, 0.0)]
        .into_iter()
        .take(binds)
        .map(|position| bind(vector(0.0, 0.0, 0.0), position))
        .collect(),
    }),
    ..visual(MODEL_TYPE_SKELETON_ANIM)
  }
}

fn floats(bones: usize) -> Vec<f32> {
  (0..bones * VisualTransform::FLOATS).map(|it| it as f32).collect()
}

#[test]
fn finds_a_bone_by_its_name_without_regard_to_case() {
  let pose: VisualRestPose = VisualSkeleton::get_rest_pose(&two_bones(2)).expect("the bind pose to resolve");

  assert_eq!(pose.find("bone_lamp").map(|it| it.c.y), Some(1.0));
  assert!(pose.find("bone_missing").is_none());
}

#[test]
fn stands_in_no_bind_pose_where_the_bind_records_do_not_match_the_bones() {
  assert!(VisualSkeleton::get_rest_pose(&two_bones(1)).is_none());
}

#[test]
fn reads_a_bake_frame_back_as_it_writes_one() {
  let names: Vec<String> = vec![String::from("root"), String::from("child")];
  let pose: VisualRestPose = VisualRestPose::of_floats(names, &floats(2)).expect("a full frame to read");

  assert_eq!(pose.transforms[1].c, vector(21.0, 22.0, 23.0));
  assert_eq!(pose.to_floats(), floats(2));
}

#[test]
fn reads_no_pose_out_of_a_frame_too_short_for_its_bones_or_out_of_nothing() {
  let names: Vec<String> = vec![String::from("root"), String::from("child")];

  assert!(VisualRestPose::of_floats(names, &floats(1)).is_none());
  assert!(VisualRestPose::of_floats(Vec::new(), &[]).is_none());
}
