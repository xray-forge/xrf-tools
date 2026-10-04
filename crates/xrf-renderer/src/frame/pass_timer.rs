use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::contract::render_pass_cost::RenderPassCost;

/// Timestamps one frame writes at most: its start and the end of each pass after it.
const CAPACITY: u32 = 64;

/// Frames of timestamps on their way back at once, so a slow map never stalls the next frame's timing.
const SLOTS: usize = 3;

/// Nothing copied, a copy recorded but not submitted, a map asked for, a map done.
const IDLE: u8 = 0;
const RECORDED: u8 = 1;
const MAPPING: u8 = 2;
const MAPPED: u8 = 3;

/// One frame's timestamps on their way back, with the pass each one ends.
struct TimerSlot {
  buffer: wgpu::Buffer,
  state: Arc<AtomicU8>,
  names: Vec<&'static str>,
}

/// What each pass of a viewport's frames cost on the GPU, timed by timestamps written into the frame's encoder between
/// its passes and read back without waiting: averaged over the frames read since the last report.
pub struct PassTimer {
  /// The frame's timestamps, `None` where the device writes none inside an encoder.
  queries: Option<wgpu::QuerySet>,
  resolve: wgpu::Buffer,
  slots: Vec<TimerSlot>,
  /// Nanoseconds a timestamp tick lasts.
  period: f32,
  is_enabled: bool,
  /// The slot this frame records into and the passes it has ended so far, while a frame is timed.
  frame: Option<(usize, Vec<&'static str>)>,
  /// Milliseconds and frames summed per pass since the last report, in the order passes were first seen.
  sums: Vec<(&'static str, f64, u32)>,
}

impl PassTimer {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let size: u64 = u64::from(CAPACITY) * 8;
    let queries: Option<wgpu::QuerySet> = device
      .features()
      .contains(wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
      .then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
          label: Some("pass timer"),
          ty: wgpu::QueryType::Timestamp,
          count: CAPACITY,
        })
      });

    Self {
      queries,
      resolve: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pass timer resolve"),
        size,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      slots: (0..SLOTS)
        .map(|_| TimerSlot {
          buffer: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pass timer readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
          }),
          state: Arc::new(AtomicU8::new(IDLE)),
          names: Vec::new(),
        })
        .collect(),
      period: queue.get_timestamp_period(),
      is_enabled: false,
      frame: None,
      sums: Vec::new(),
    }
  }

  /// Whether frames are timed: asked for, and the device writes timestamps inside an encoder.
  pub fn is_timing(&self) -> bool {
    self.is_enabled && self.queries.is_some()
  }

  pub fn set_enabled(&mut self, is_enabled: bool) {
    if self.is_enabled != is_enabled {
      self.is_enabled = is_enabled;
      self.sums.clear();
    }
  }

  /// Starts timing a frame, unless timing is off or every slot is still on its way back.
  pub fn begin(&mut self, encoder: &mut wgpu::CommandEncoder) {
    self.frame = None;

    let Some(queries) = self.queries.as_ref().filter(|_| self.is_enabled) else {
      return;
    };
    let Some(slot) = self
      .slots
      .iter()
      .position(|slot| slot.state.load(Ordering::Acquire) == IDLE)
    else {
      return;
    };

    encoder.write_timestamp(queries, 0);
    self.frame = Some((slot, Vec::new()));
  }

  /// Ends a pass of the frame being timed: what the GPU did since the last mark is that pass's.
  pub fn mark(&mut self, encoder: &mut wgpu::CommandEncoder, name: &'static str) {
    if let (Some(queries), Some((_, names))) = (&self.queries, &mut self.frame)
      && (names.len() as u32) + 1 < CAPACITY
    {
      names.push(name);
      encoder.write_timestamp(queries, names.len() as u32);
    }
  }

  /// Resolves the frame's timestamps into its slot, to be read back once the frame is done.
  pub fn finish(&mut self, encoder: &mut wgpu::CommandEncoder) {
    let (Some(queries), Some((index, names))) = (&self.queries, self.frame.take()) else {
      return;
    };

    if names.is_empty() {
      return;
    }

    let count: u32 = names.len() as u32 + 1;
    let slot: &mut TimerSlot = &mut self.slots[index];

    encoder.resolve_query_set(queries, 0..count, &self.resolve, 0);
    encoder.copy_buffer_to_buffer(&self.resolve, 0, &slot.buffer, 0, u64::from(count) * 8);
    slot.names = names;
    slot.state.store(RECORDED, Ordering::Release);
  }

  /// Asks for the timestamps recorded with the frame just submitted.
  pub fn request(&self) {
    for slot in &self.slots {
      if slot.state.load(Ordering::Acquire) == RECORDED {
        let state: Arc<AtomicU8> = Arc::clone(&slot.state);

        slot.state.store(MAPPING, Ordering::Release);
        slot.buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
          state.store(if result.is_ok() { MAPPED } else { IDLE }, Ordering::Release);
        });
      }
    }
  }

  /// Each pass's mean GPU milliseconds over the frames read back since the last call, which starts the next span.
  pub fn take(&mut self) -> Vec<RenderPassCost> {
    self.collect();

    self
      .sums
      .drain(..)
      .map(|(name, total, frames)| RenderPassCost {
        name: name.to_owned(),
        gpu_time: (total / f64::from(frames.max(1))) as f32,
      })
      .collect()
  }

  /// Adds every frame read back to the sums, freeing its slot; called each frame, so the slots keep turning over between
  /// reports.
  pub fn collect(&mut self) {
    let to_milliseconds: f64 = f64::from(self.period) / 1_000_000.0;

    for slot in &mut self.slots {
      if slot.state.load(Ordering::Acquire) != MAPPED {
        continue;
      }

      if let Ok(view) = slot.buffer.slice(..).get_mapped_range() {
        let stamps: Vec<u64> = view[..(slot.names.len() + 1) * 8]
          .as_chunks::<8>()
          .0
          .iter()
          .map(|bytes| u64::from_le_bytes(*bytes))
          .collect();

        for (index, name) in slot.names.iter().enumerate() {
          // A tick count running backwards across a pass is a driver's, not a negative cost.
          let spent: f64 = stamps[index + 1].saturating_sub(stamps[index]) as f64 * to_milliseconds;

          match self.sums.iter_mut().find(|(it, ..)| it == name) {
            Some((_, total, frames)) => {
              *total += spent;
              *frames += 1;
            }
            None => self.sums.push((name, spent, 1)),
          }
        }
      }

      slot.buffer.unmap();
      slot.state.store(IDLE, Ordering::Release);
    }
  }
}
