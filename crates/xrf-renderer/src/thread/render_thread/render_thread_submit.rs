use std::time::{Duration, Instant};

use xrf_error::XrfError;
use xrf_renderer_core::{ExecutedGraph, ExecutedGroup, GraphRuntime};

use wgpu::util::StagingBelt;

use crate::capture::burst::burst_shot::BurstShot;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::frame_phases::FramePhases;
use crate::frame::frame_span_timer::FrameSpanTimer;
use crate::host::render_scene_id::RenderSceneId;
use crate::thread::gpu_state::GpuState;
use crate::viewport::pick_in_flight::PickInFlight;
use crate::window::window_frame::WindowFrame;

use super::{BurstCopy, RenderThread, ShotCopy, ViewFrame};

impl RenderThread {
  /// Gives up a frame its graph could not draw: what the frame's own encoder holds is not of the frame (the uploads and
  /// the acceleration structures the scenes built, which they count as done) and still goes up; what the graph copied
  /// out for reading back is dropped, its picks wait for the next frame, and its captures and bursts are refused, a
  /// burst's frames no longer following one another.
  pub(super) fn abandon_frame(
    &mut self,
    (gpu, kept): (&GpuState, &mut Option<StagingBelt>),
    encoder: FrameEncoder,
    (frames, firsts): (Vec<ViewFrame>, &[(RenderSceneId, RenderViewportId)]),
    captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)>,
    error: &XrfError,
  ) {
    log::error!("The frame cannot be drawn: {error}");

    let scope: wgpu::ErrorScopeGuard = gpu.context.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

    gpu.context.queue.submit([commands]);
    *kept = Some(belt);

    if let Some(error) = pollster::block_on(scope.pop()) {
      log::error!("The frame's uploads cannot be submitted: {error}");
    }

    let message: String = error.to_string();

    for (id, _, pick) in frames {
      let Some(viewport) = self.viewports.get_mut(&id) else {
        continue;
      };

      if let Some(level) = viewport.get_drawn_view_mut() {
        level.abandon_frame();
      }

      if let Some((pick, _)) = pick {
        viewport.picks.insert(0, pick);
      }

      viewport.stop_burst(&message);
    }

    for (scene_id, _) in firsts {
      if let Some(scene) = self.scenes.get_mut(scene_id) {
        scene.lights.abandon_shadows();
        scene.impostors.abandon_frame();
      }
    }

    for (_, _, reply) in captures {
      let _ = reply.send(Err(XrfError::new_invalid_error(message.clone())));
    }
  }

  /// Submits the frame's own encoder, the graph's command buffers and the frame's span, timing the phases from
  /// `recording` on, the windows' acquire waits (`waited`) apart; asks for the timers back. Answers the graph's encode
  /// groups as executed.
  pub(super) fn submit_frame(
    &mut self,
    (gpu, runtime, frame_span, kept): (
      &GpuState,
      &mut GraphRuntime,
      &mut FrameSpanTimer,
      &mut Option<StagingBelt>,
    ),
    executed: ExecutedGraph,
    encoder: FrameEncoder,
    (recording, waited): (Instant, Duration),
    phases: &mut FramePhases,
  ) -> Vec<ExecutedGroup> {
    let device: &wgpu::Device = &gpu.context.device;
    let finishing: Instant = Instant::now();
    let mut commands: Vec<wgpu::CommandBuffer> = {
      profiling::scope!("finish", "frame");

      let (commands, belt): (wgpu::CommandBuffer, StagingBelt) = encoder.finish();

      *kept = Some(belt);

      vec![commands]
    };
    let encoded: Duration = finishing.elapsed() + executed.encode;

    phases.record += recording.elapsed().saturating_sub(encoded + waited);
    phases.encode += encoded;
    commands.extend(executed.commands);
    commands.extend(frame_span.end(device));

    let submitting: Instant = Instant::now();

    {
      profiling::scope!("submit");

      gpu.context.queue.submit(commands);
    }
    phases.submit = submitting.elapsed();
    runtime.timer.request();

    frame_span.request();

    executed.groups
  }

  /// Ends each view's submitted frame: its history kept, its stats and its pick asked for back, its burst moved on with
  /// the frame copied out of it; and each capture asked for back.
  pub(super) fn end_view_frames(
    &mut self,
    frames: Vec<ViewFrame>,
    captures: Vec<(RenderViewportId, FrameCapture, CaptureReply)>,
    (mut shots, mut bursts): (Vec<ShotCopy>, Vec<BurstCopy>),
  ) {
    for (id, frame, pick) in frames {
      let Some(viewport) = self.viewports.get_mut(&id) else {
        continue;
      };

      if let Some(burst) = viewport.burst.as_mut().filter(|burst| burst.is_drawing()) {
        let kept: Option<BurstShot> = bursts
          .iter()
          .position(|(shot, _)| *shot == id)
          .map(|index| bursts.swap_remove(index).1);

        burst.advance(kept);
      }

      // A capture's frame moves it on, with the shot copied out of it, if any.
      if let Some(capture) = viewport.capture.as_mut().filter(|capture| !capture.is_ended()) {
        let kept: Option<FrameCapture> = shots
          .iter()
          .position(|(shot, _)| *shot == id)
          .map(|index| shots.swap_remove(index).1);

        capture.advance(kept);

        continue;
      }

      let Some((_, level)) = viewport.shown.as_mut() else {
        continue;
      };

      level.end_frame();
      level.request_stats();

      if let (Some(slot), Some((pick, unprojection))) = (frame.pick_slot, pick) {
        level.request_pick(slot);
        viewport.picks_in_flight.push(PickInFlight {
          pick,
          slot,
          unprojection,
          frame: viewport.frame,
        });
      }
    }

    for (id, capture, reply) in captures {
      capture.request();

      if let Some(viewport) = self.viewports.get_mut(&id) {
        viewport.captures_in_flight.push((capture, reply));
      }
    }
  }

  /// Presents each window, and notes the frame in its viewports with its own acquire, present and interval: `cpu` the
  /// frame's time on the render thread, the acquire waits apart, `span` the GPU's latest read back. A limited rate
  /// presents at the frame's due however long its work took, so frames are shown evenly; how long this one took from
  /// its start sets how far ahead of its due the next starts.
  pub(super) fn present_windows(
    &mut self,
    queue: &wgpu::Queue,
    windows: Vec<WindowFrame>,
    phases: FramePhases,
    (cpu, span): (Duration, Option<f32>),
    groups: &[ExecutedGroup],
  ) {
    self.pacer.note_ready(self.last_frame.elapsed());
    self.pacer.wait_for_due();

    for window in windows {
      let Some(image) = window.image else {
        continue;
      };
      let presenting: Instant = Instant::now();

      {
        profiling::scope!("present");

        queue.present(image);
      }

      let presented: Instant = Instant::now();
      let window_phases: FramePhases = FramePhases {
        acquire: window.acquire,
        present: presented.duration_since(presenting),
        ..phases
      };
      // Between present calls, as PresentMon and the overlays built on it measure a frame, so how long a call blocks
      // does not read as uneven pacing.
      let interval: Duration = self
        .last_present
        .insert(window.key, presenting)
        .map_or(Duration::ZERO, |last| presenting.duration_since(last));

      for (id, _, _) in &window.drawn {
        if let Some(viewport) = self.viewports.get_mut(id) {
          viewport.frame += 1;
          viewport.record_frame((interval, cpu, span), &window_phases, groups);
        }
      }
    }
  }
}
