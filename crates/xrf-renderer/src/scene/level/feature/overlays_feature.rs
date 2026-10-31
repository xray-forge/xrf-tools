use glam::Vec3;

use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_overlay_segment::RenderOverlaySegment;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::frame::frame_encoder::FrameEncoder;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::static_scene::static_object_box::StaticObjectBox;
use crate::scene::static_scene::static_selection::StaticSelection;

/// What a view draws over its frame and marks in it: the page's overlays with a selected spawned object's box, and the
/// selection resolved against the scene.
#[derive(Default)]
pub struct OverlaysFeature {
  overlays: Option<LevelOverlays>,
  /// The selection box the overlays were made with, so a box that moved or came into the scene makes them again.
  overlays_box: Option<RenderOverlay>,
  /// What the selection marks, for the target it was resolved for and the scene's contents it was resolved with.
  selection: Option<(RenderSelectionTarget, usize, Option<StaticSelection>)>,
  /// The colour the selection is outlined in this frame, where something is selected and resolved.
  selection_color: Option<[f32; 3]>,
}

impl OverlaysFeature {
  /// Makes what it draws over its frame where the page's overlays or the selection's box changed: the page's, then a
  /// selected spawned object's box.
  pub fn set(
    &mut self,
    (scene, device): (&LevelScene, &wgpu::Device),
    (overlays, version): (&[RenderOverlay], u64),
    selection: Option<&RenderSelection>,
  ) {
    let boxed: Option<RenderOverlay> = selection.and_then(|selection| match selection.target {
      RenderSelectionTarget::Spawn { object } => scene
        .statics
        .get_object_box(object)
        .map(|object_box| Self::to_box_lines(object_box, selection.color)),
      RenderSelectionTarget::Surface { .. } => None,
    });

    if self.overlays.as_ref().is_none_or(|it| it.version != version) || self.overlays_box != boxed {
      let all: Vec<RenderOverlay> = overlays.iter().cloned().chain(boxed.clone()).collect();

      self.overlays = Some(LevelOverlays::new(device, &all, version));
      self.overlays_box = boxed;
    }
  }

  /// Follows the scene's skeletons where the overlays draw them.
  pub fn follow_skeletons(&mut self, (scene, device, encoder): (&LevelScene, &wgpu::Device, &mut FrameEncoder)) {
    if let Some(overlays) = self.overlays.as_mut().filter(|it| it.skeleton.is_some()) {
      overlays.set_skeleton((device, encoder), &scene.skeleton_segments);
    }
  }

  /// What a selection marks in the scene now, resolved again only once the target or the scene changed; none where
  /// nothing is selected or what it names is not in the scene yet.
  pub fn resolve_selection(
    &mut self,
    scene: &LevelScene,
    selection: Option<&RenderSelection>,
  ) -> Option<&StaticSelection> {
    self.selection_color = None;

    let selection: &RenderSelection = selection?;
    let contents: usize = scene.statics.get_contents();

    if self
      .selection
      .as_ref()
      .is_none_or(|(target, at, _)| *target != selection.target || *at != contents)
    {
      self.selection = Some((
        selection.target,
        contents,
        scene.statics.resolve_selection(&selection.target),
      ));
    }

    let resolved: &StaticSelection = self.selection.as_ref()?.2.as_ref()?;

    self.selection_color = Some(selection.color);

    Some(resolved)
  }

  /// The colour the selection is outlined in this frame, where something is selected and resolved.
  pub fn get_selection_color(&self) -> Option<[f32; 3]> {
    self.selection_color
  }

  /// What it draws over its frame, once made.
  pub fn get_overlays(&self) -> Option<&LevelOverlays> {
    self.overlays.as_ref()
  }

  /// A box's twelve edges as lines in one colour, seen through what is in front.
  fn to_box_lines(object_box: StaticObjectBox, color: [f32; 3]) -> RenderOverlay {
    let segments: Vec<RenderOverlaySegment> = object_box
      .to_edges()
      .map(|[from, to]: [Vec3; 2]| RenderOverlaySegment {
        ends: [from.to_array(), to.to_array()],
        colors: [color; 2],
      })
      .collect();

    RenderOverlay::Lines {
      segments,
      is_depth_tested: false,
    }
  }
}

impl ViewFeature for OverlaysFeature {
  /// The overlays' lines, points, sun and skeleton.
  fn get_memory(&self) -> u64 {
    self.overlays.as_ref().map_or(0, |overlays| {
      [&overlays.lines, &overlays.points, &overlays.skeleton_lines]
        .into_iter()
        .flatten()
        .map(|(buffer, _)| buffer.size())
        .chain(overlays.sun.as_ref().map(wgpu::Buffer::size))
        .sum()
    })
  }

  fn release(&mut self) {
    self.overlays = None;
    self.overlays_box = None;
    self.selection = None;
  }
}
