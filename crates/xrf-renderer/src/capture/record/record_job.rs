use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Instant;

use xrf_error::XrfError;

use crate::capture::record::record_event::RecordEvent;
use crate::capture::record::record_frame::RecordFrame;
use crate::capture::record::record_layout::RecordLayout;
use crate::capture::record::record_shot::RecordShot;
use crate::frame::readback::gpu_readback::GpuReadback;

/// Where a buffer of the ring stands: free, copied into by the frame being drawn, asked back with its frame's present
/// time once that is known, or with the recorder.
#[derive(Clone, Copy, Debug, PartialEq)]
enum RecordSlot {
  Free,
  Drawn { index: u64 },
  Asked { index: u64, presented: Option<Instant> },
  Sent,
}

/// A viewport recorded as it is presented, alongside what it draws: a frame kept a sixtieth of a second on from the
/// last, presented at the size locked at the start and copied into one of a ring of four buffers, which the recorder
/// reads on its own thread. With every buffer on its way or held by the recorder, a frame due is dropped and counted,
/// so the render thread never waits on a recording.
pub struct RecordJob {
  layout: RecordLayout,
  readbacks: Vec<Arc<GpuReadback>>,
  slots: [RecordSlot; Self::RING],
  /// When the first frame was due, which the sixtieths are counted from, and the last one taken or dropped.
  started: Option<Instant>,
  last: Option<u64>,
  dropped: u64,
  events: Sender<RecordEvent>,
  is_ended: bool,
}

impl RecordJob {
  /// Buffers a recording's frames go back through.
  pub const RING: usize = 4;

  /// A recording of frames `size` pixels across, told to `events`.
  pub fn new(size: (u32, u32), events: Sender<RecordEvent>) -> Self {
    Self {
      layout: RecordLayout::new(size),
      readbacks: Vec::new(),
      slots: [RecordSlot::Free; Self::RING],
      started: None,
      last: None,
      dropped: 0,
      events,
      is_ended: false,
    }
  }

  /// The frame this frame draws at `now`, where one is due a sixtieth on from the last and a buffer is free; a frame
  /// due with none free is dropped and counted. The buffers are made with the first.
  pub fn take_shot(&mut self, device: &wgpu::Device, now: Instant) -> Option<RecordShot> {
    if self.is_ended {
      return None;
    }

    // Counted to the nearest sixtieth, so a frame presented a little early or late keeps its place.
    let started: Instant = *self.started.get_or_insert(now);
    let index: u64 = (now.duration_since(started).as_secs_f64() / RecordFrame::INTERVAL.as_secs_f64()).round() as u64;

    if self.last.is_some_and(|last| index <= last) {
      return None;
    }

    self.last = Some(index);

    if self.readbacks.is_empty() {
      let size: u64 = self.layout.get_buffer_size();

      self.readbacks = (0..Self::RING)
        .map(|_| Arc::new(GpuReadback::new(device, "record frame", size)))
        .collect();
    }

    let Some(slot) = (0..Self::RING).find(|slot| self.is_free(*slot)) else {
      self.dropped += 1;

      return None;
    };

    self.slots[slot] = RecordSlot::Drawn { index };

    Some(RecordShot::new(slot, Arc::clone(&self.readbacks[slot]), self.layout))
  }

  /// Ends the frame `shot` was drawn in, submitted: its copy asked back, or its buffer freed where the frame drew none.
  pub fn advance(&mut self, shot: RecordShot) {
    let readback: &GpuReadback = &self.readbacks[shot.slot];

    self.slots[shot.slot] = match self.slots[shot.slot] {
      RecordSlot::Drawn { index } if readback.is_recorded() => {
        readback.request();

        RecordSlot::Asked { index, presented: None }
      }
      _ => {
        readback.abandon();

        RecordSlot::Free
      }
    };
  }

  /// Frees the buffer of `shot`, drawn in a frame that was never submitted.
  pub fn abandon(&mut self, shot: &RecordShot) {
    self.readbacks[shot.slot].abandon();
    self.slots[shot.slot] = RecordSlot::Free;
  }

  /// Notes `at` as the present time of the frames asked back that had none, the frame just presented.
  pub fn present(&mut self, at: Instant) {
    for slot in &mut self.slots {
      if let RecordSlot::Asked { presented, .. } = slot {
        presented.get_or_insert(at);
      }
    }
  }

  /// Sends the recorder each frame back since and presented, and takes back the buffers it read or let go.
  pub fn poll(&mut self) {
    for slot in 0..self.slots.len() {
      match self.slots[slot] {
        RecordSlot::Asked {
          index,
          presented: Some(presented),
        } if self.readbacks[slot].is_back() => {
          self.slots[slot] = RecordSlot::Sent;

          let frame: RecordFrame = RecordFrame::new(
            Arc::clone(&self.readbacks[slot]),
            self.layout,
            index,
            presented,
            self.dropped,
          );

          if self.events.send(RecordEvent::Frame(frame)).is_err() {
            self.is_ended = true;
          }
        }
        RecordSlot::Sent if self.readbacks[slot].is_free() => self.slots[slot] = RecordSlot::Free,
        _ => {}
      }
    }
  }

  /// Frames dropped so far, every buffer on its way or held.
  #[cfg(test)]
  pub fn get_dropped(&self) -> u64 {
    self.dropped
  }

  /// Stops it, telling the recorder why; nothing more is kept.
  pub fn stop(&mut self, error: XrfError) {
    if !self.is_ended {
      self.is_ended = true;
      let _ = self.events.send(RecordEvent::Stopped { error });
    }
  }

  /// Whether it stopped or its recorder went.
  pub fn is_ended(&self) -> bool {
    self.is_ended
  }

  fn is_free(&self, slot: usize) -> bool {
    self.slots[slot] == RecordSlot::Free && self.readbacks[slot].is_free()
  }
}
