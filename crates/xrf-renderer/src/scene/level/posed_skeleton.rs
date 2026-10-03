use glam::{Mat4, Vec3, Vec4};

use crate::host::render_model_skeleton::RenderModelSkeleton;
use crate::host::render_motion::RenderMotion;

/// Floats one bone's transform takes: its basis' three columns, then its translation.
const FLOATS_PER_BONE: usize = 12;

/// One skinned object's skeleton as the renderer poses it: its binds inverted once, the matrices it stood by the
/// frame before, and where its joints stand now, for the overlay.
pub struct PosedSkeleton {
  bones: usize,
  binds: Vec<f32>,
  inverse_binds: Vec<Mat4>,
  pairs: Vec<(u16, u16)>,
  /// The rows the last frame drew with, which this frame's motion is measured from.
  previous: Option<Vec<[Vec4; 3]>>,
  /// Each joint's position this frame, in model space.
  joints: Vec<Vec3>,
}

impl PosedSkeleton {
  pub fn new(skeleton: &RenderModelSkeleton) -> Self {
    let bones: usize = skeleton.binds.len() / FLOATS_PER_BONE;

    Self {
      bones,
      inverse_binds: (0..bones)
        .map(|bone| to_matrix(&skeleton.binds, bone * FLOATS_PER_BONE).inverse())
        .collect(),
      binds: skeleton.binds.clone(),
      pairs: skeleton.pairs.clone(),
      previous: None,
      joints: Vec::new(),
    }
  }

  /// This frame's skin matrices as three rows a bone, from each bind to where `motion`'s `frame` stands it, or the bind
  /// pose where the frame lies outside it; a hidden bone collapses everything hanging from it to the origin. Gives the
  /// last frame's rows beside them, this frame's again on the first.
  pub fn pose(
    &mut self,
    motion: Option<&RenderMotion>,
    frame: u32,
    hidden: &[u32],
  ) -> (Vec<[Vec4; 3]>, Vec<[Vec4; 3]>) {
    let stride: usize = self.bones * FLOATS_PER_BONE;
    let frame_start: usize = frame as usize * stride;
    let source: (&[f32], usize) = match motion {
      Some(motion)
        if motion.bones as usize == self.bones
          && frame < motion.frames
          && motion.transforms.len() >= frame_start + stride =>
      {
        (&motion.transforms, frame_start)
      }
      _ => (&self.binds, 0),
    };
    let posed: Vec<Mat4> = (0..self.bones)
      .map(|bone| to_matrix(source.0, source.1 + bone * FLOATS_PER_BONE))
      .collect();
    let current: Vec<[Vec4; 3]> = posed
      .iter()
      .enumerate()
      .map(|(bone, matrix)| {
        let skin: Mat4 = if hidden.contains(&(bone as u32)) {
          Mat4::ZERO
        } else {
          *matrix * self.inverse_binds[bone]
        };

        [skin.row(0), skin.row(1), skin.row(2)]
      })
      .collect();
    let previous: Vec<[Vec4; 3]> = self
      .previous
      .replace(current.clone())
      .unwrap_or_else(|| current.clone());

    // The overlay keeps drawing hidden bones: it is the only thing left saying where a hidden part sits.
    self.joints = posed.iter().map(|matrix| matrix.w_axis.truncate()).collect();

    (current, previous)
  }

  /// The segments the overlay draws, child then parent, where this frame stands the joints, in model space.
  pub fn list_segments(&self) -> Vec<(Vec3, Vec3)> {
    self
      .pairs
      .iter()
      .filter_map(|(child, parent)| Some((*self.joints.get(*child as usize)?, *self.joints.get(*parent as usize)?)))
      .collect()
  }
}

/// A bone's twelve floats as a matrix: its basis' three columns, then its translation.
fn to_matrix(values: &[f32], at: usize) -> Mat4 {
  let read = |offset: usize| -> Vec3 {
    Vec3::new(
      values.get(at + offset).copied().unwrap_or(0.0),
      values.get(at + offset + 1).copied().unwrap_or(0.0),
      values.get(at + offset + 2).copied().unwrap_or(0.0),
    )
  };

  Mat4::from_cols(
    read(0).extend(0.0),
    read(3).extend(0.0),
    read(6).extend(0.0),
    read(9).extend(1.0),
  )
}
