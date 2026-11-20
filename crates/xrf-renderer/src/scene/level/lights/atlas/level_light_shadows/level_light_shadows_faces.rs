use std::collections::HashSet;

use glam::Vec4;

use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::lights::atlas::level_light_shadows::{
  FACE_BUDGET, FACE_PASSES, FACE_SEED, FaceCandidate, FaceDraw, FaceDrawn, FaceDue, FaceKey, FaceOffer, FaceSet,
  LevelLightShadows,
};
use crate::scene::level::lights::atlas::light_shadow_entry::LightShadowEntry;
use crate::scene::level::lights::atlas::light_shadow_face::LightShadowFace;
use crate::scene::level::lights::atlas::light_shadow_set::LightShadowSet;
use crate::scene::level::shadow::shadow_frame::ShadowFrame;

impl LevelLightShadows {
  /// Offers `scheduler` the faces wanting a draw this frame, the nearest lights' first, each once however many views
  /// asked for it, by their texels: one holding nothing yet or changed under within `STALE_WAIT_MOST` frames, one
  /// swaying within `SWAY_WAIT_MOST`, one its light carried at once.
  pub(crate) fn finish(&mut self, scheduler: &mut PeriodicScheduler) {
    self.candidates.sort_by(|a, b| {
      (a.order.is_away, a.order.is_current)
        .cmp(&(b.order.is_away, b.order.is_current))
        .then(a.order.by.compare(b.order.by))
    });
    let mut queued: HashSet<FaceKey> = HashSet::new();
    let faces: Vec<FaceCandidate> = self
      .candidates
      .iter()
      .filter(|candidate| queued.insert(candidate.key))
      .take(FACE_BUDGET)
      .copied()
      .collect();

    self.queue.clear();

    for FaceCandidate { key, wait_most, .. } in faces {
      let Some(state) = self.get_face(key) else {
        continue;
      };
      let texels: f32 = (state.tile.size as f32).powi(2);
      let job: PeriodicJob = if wait_most == 0 {
        PeriodicJob::new_now(&FACE_PASSES, FACE_SEED, texels)
      } else {
        PeriodicJob::new_waiting(&FACE_PASSES, FACE_SEED, texels, state.waited, wait_most)
      };

      self.queue.push(FaceOffer {
        key,
        ticket: scheduler.offer(job),
      });
    }
  }

  /// Readies the faces `scheduler` granted this frame to be culled into their slots and drawn into their squares of the
  /// atlas, one not granted waiting a frame more, and lets a light's new faces take the place of its old once all are
  /// drawn; `encoder` takes the copies a list growing makes.
  pub(crate) fn prepare(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    let scene = frame.scene;
    let contents: usize = scene.get_contents();

    self.due.clear();
    self.drawn.clear();
    self.hold_atlas(device);
    self.hold_statics(device);

    if self.queue.is_empty() {
      return;
    }

    self.region = scene.cull.get_list_bytes().next_multiple_of(self.alignment);
    self.lists.reserve(device, encoder, self.region * FACE_BUDGET as u64);

    for FaceOffer { key, ticket } in std::mem::take(&mut self.queue) {
      let slot: usize = self.due.len();
      let is_granted: bool = scheduler.is_granted(ticket);
      let Some(entry) = self.entries.get_mut(&key.handle) else {
        continue;
      };
      // What sways in its reach, trees or a kind's leaning casters, splits it into what stands still, kept in its static
      // tile, and what moves over its copy: drawing it whole again for the grass alone redrew every caster.
      let is_split: bool = self.statics.is_some() && (entry.swaying > 0.0 || entry.casting_lean > 0.0);
      let casting: DrawableKinds = entry.casting;
      let set: Option<&mut LightShadowSet> = match key.set {
        FaceSet::Shown => entry.shown.as_mut(),
        FaceSet::Next => entry.next.as_mut(),
      };
      let Some(state) = set.and_then(|set| set.faces.get_mut(key.face)) else {
        continue;
      };

      if !is_granted {
        state.waited += 1;

        continue;
      }

      state.waited = 0;

      let view: &ViewBinding = &self.views[slot];

      view.write(
        encoder,
        &CameraUniform::new(
          &state.view,
          RenderRect {
            x: 0,
            y: 0,
            width: state.tile.size,
            height: state.tile.size,
          },
          Vec4::ZERO,
        ),
      );
      let draw: FaceDraw = match (is_split, state.drawn_static) {
        (false, _) => FaceDraw::Whole,
        (true, None) => FaceDraw::Static,
        (true, Some(_)) => FaceDraw::Swaying,
      };

      state.drawn = Some(contents);
      state.drawn_at = frame.sway.time;
      state.casting = casting;

      if draw == FaceDraw::Static {
        state.drawn_static = Some(contents);
      }

      self.due.push(FaceDue {
        slot,
        tile: state.tile,
        draw,
        casting,
      });
      self.drawn.push(FaceDrawn {
        handle: key.handle,
        tile: state.tile,
      });
    }

    // A light's new faces, all drawn, take the place of its old.
    for entry in self.entries.values_mut() {
      if entry.next.as_ref().is_some_and(LightShadowSet::is_drawn) {
        if let Some(shown) = entry.shown.take() {
          Self::release_set(&mut self.allocator, shown);
        }

        entry.shown = entry.next.take();
      }
    }
  }

  /// A face of a light's shown set or its next.
  fn get_face(&self, FaceKey { handle, set, face }: FaceKey) -> Option<&LightShadowFace> {
    let entry: &LightShadowEntry = self.entries.get(&handle)?;
    let set: &LightShadowSet = match set {
      FaceSet::Shown => entry.shown.as_ref(),
      FaceSet::Next => entry.next.as_ref(),
    }?;

    set.faces.get(face)
  }
}
