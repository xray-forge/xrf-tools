use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::graph::timing::graph_pass_time::GraphPassTime;
use crate::graph::timing::timer_slot::TimerSlot;

/// What each pass of the graph's frames cost on the GPU, with no code in any pass: the graph writes a timestamp at the
/// start of each encode group and after each pass or render pass, and the stamps are read back without waiting, a few
/// frames in flight at once. Timing needs timestamps inside encoders; a device without them times nothing.
pub struct GraphTimer {
  queries: Option<wgpu::QuerySet>,
  resolve: wgpu::Buffer,
  slots: Vec<TimerSlot>,
  /// Nanoseconds a timestamp tick lasts.
  period: f32,
  is_enabled: bool,
  /// The slot this frame records into and what its stamps so far end, while a frame is timed.
  frame: Option<(usize, Vec<Option<String>>)>,
  /// Milliseconds and frames summed per pass since the last `take`, in the order passes were first seen.
  sums: Vec<(String, f64, u32)>,
}

impl GraphTimer {
  /// Stamps a frame writes at most.
  pub const CAPACITY: u32 = 256;
  /// Frames of stamps on their way back at once.
  pub const SLOTS: usize = 3;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let size: u64 = u64::from(Self::CAPACITY) * 8;
    let queries: Option<wgpu::QuerySet> = device
      .features()
      .contains(wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
      .then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
          label: Some("graph timer"),
          ty: wgpu::QueryType::Timestamp,
          count: Self::CAPACITY,
        })
      });

    Self {
      queries,
      resolve: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("graph timer resolve"),
        size,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      slots: (0..Self::SLOTS)
        .map(|_| TimerSlot {
          buffer: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph timer readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
          }),
          state: Arc::new(AtomicU8::new(TimerSlot::IDLE)),
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

  /// Asks for the stamps recorded with the frame just submitted; call after each submit.
  pub fn request(&self) {
    for slot in &self.slots {
      if slot.state.load(Ordering::Acquire) == TimerSlot::RECORDED {
        let state: Arc<AtomicU8> = Arc::clone(&slot.state);

        slot.state.store(TimerSlot::MAPPING, Ordering::Release);
        slot.buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
          state.store(
            if result.is_ok() {
              TimerSlot::MAPPED
            } else {
              TimerSlot::IDLE
            },
            Ordering::Release,
          );
        });
      }
    }
  }

  /// Each pass's mean GPU milliseconds over the frames read back since the last call, which starts the next span.
  pub fn take(&mut self) -> Vec<GraphPassTime> {
    self.collect();

    self
      .sums
      .drain(..)
      .map(|(name, total, frames)| GraphPassTime {
        name,
        gpu_time: (total / f64::from(frames.max(1))) as f32,
      })
      .collect()
  }

  /// Starts timing a frame, unless timing is off or every slot is still on its way back; answers whether it does.
  pub(crate) fn begin_frame(&mut self) -> bool {
    self.collect();
    self.frame = None;

    if !self.is_timing() {
      return false;
    }

    self.frame = self
      .slots
      .iter()
      .position(|slot| slot.state.load(Ordering::Acquire) == TimerSlot::IDLE)
      .map(|index| (index, Vec::new()));

    self.frame.is_some()
  }

  /// Writes a stamp ending `name`, or starting an encode group where `None`; nothing past the capacity.
  pub(crate) fn stamp(&mut self, encoder: &mut wgpu::CommandEncoder, name: Option<String>) {
    if let (Some(queries), Some((_, names))) = (&self.queries, &mut self.frame)
      && (names.len() as u32) < Self::CAPACITY
    {
      encoder.write_timestamp(queries, names.len() as u32);
      names.push(name);
    }
  }

  /// Resolves the frame's stamps into its slot, from the last encode group's encoder.
  pub(crate) fn finish_frame(&mut self, encoder: &mut wgpu::CommandEncoder) {
    let (Some(queries), Some((index, names))) = (&self.queries, self.frame.take()) else {
      return;
    };

    if names.is_empty() {
      return;
    }

    let count: u32 = names.len() as u32;
    let slot: &mut TimerSlot = &mut self.slots[index];

    encoder.resolve_query_set(queries, 0..count, &self.resolve, 0);
    encoder.copy_buffer_to_buffer(&self.resolve, 0, &slot.buffer, 0, u64::from(count) * 8);
    slot.names = names;
    slot.state.store(TimerSlot::RECORDED, Ordering::Release);
  }

  /// Adds every frame read back to the sums, freeing its slot.
  fn collect(&mut self) {
    let to_milliseconds: f64 = f64::from(self.period) / 1_000_000.0;

    for slot in &mut self.slots {
      if slot.state.load(Ordering::Acquire) != TimerSlot::MAPPED {
        continue;
      }

      if let Ok(view) = slot.buffer.slice(..).get_mapped_range() {
        let stamps: Vec<u64> = view[..slot.names.len() * 8]
          .as_chunks::<8>()
          .0
          .iter()
          .map(|bytes| u64::from_le_bytes(*bytes))
          .collect();

        for (index, name) in slot.names.iter().enumerate() {
          let Some(name) = name.as_ref().filter(|_| index > 0) else {
            continue;
          };
          // A tick count running backwards across a pass is a driver's, not a negative cost.
          let spent: f64 = stamps[index].saturating_sub(stamps[index - 1]) as f64 * to_milliseconds;

          match self.sums.iter_mut().find(|(it, ..)| it == name) {
            Some((_, total, frames)) => {
              *total += spent;
              *frames += 1;
            }
            None => self.sums.push((name.clone(), spent, 1)),
          }
        }
      }

      slot.buffer.unmap();
      slot.state.store(TimerSlot::IDLE, Ordering::Release);
    }
  }
}
