use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::graph::timing::gpu_zones::GpuZones;
use crate::graph::timing::graph_pass_time::GraphPassTime;
use crate::graph::timing::graph_tagged_time::GraphTaggedTime;
use crate::graph::timing::group_stamps::GroupStamps;
use crate::graph::timing::timer_slot::{TimedPass, TimerSlot};

/// What each pass of the graph's frames cost on the GPU, with no code in any pass: the graph writes a timestamp at the
/// start of each encode group and after each pass or render pass, each group into a range of the query set of its own
/// so the groups still record at once, and the stamps are read back without waiting, a few frames in flight at once.
/// Timing needs timestamps inside encoders; a device without them times nothing.
pub struct GraphTimer {
  queries: Option<wgpu::QuerySet>,
  resolve: wgpu::Buffer,
  slots: Vec<TimerSlot>,
  /// Stamps a frame can write.
  capacity: u32,
  /// Stamps each group is given past one a pass, for the stages a pass marks of its own; more once a group overflowed.
  mark_room: u32,
  /// Nanoseconds a timestamp tick lasts.
  period: f32,
  is_enabled: bool,
  /// The slot this frame records into and each group's first stamp and how many it has, while a frame is timed.
  frame: Option<(usize, Vec<(u32, u32)>)>,
  /// Milliseconds summed per owner's pass since that owner's last `take`, in the order passes were first seen.
  sums: Vec<(u32, String, f64)>,
  /// Frames read back per owner since its last `take`, which its passes' sums are averaged over.
  frames: Vec<(u32, u32)>,
  /// The tag the frame timed next carries.
  tag: u64,
  /// Each pass of each frame read back since the last `take_tagged`, under its frame's tag, the latest `TAGGED_MOST`.
  tagged: Vec<GraphTaggedTime>,
  zones: GpuZones,
}

impl GraphTimer {
  /// Stamps a frame can write before the timer first grows.
  pub const CAPACITY: u32 = 256;
  /// Frames of stamps on their way back at once.
  pub const SLOTS: usize = 3;
  /// Passes' times kept for `take_tagged` at most, the oldest let go past it.
  pub const TAGGED_MOST: usize = 4096;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let (queries, resolve, slots) = Self::create(device, Self::CAPACITY);

    Self {
      queries,
      resolve,
      slots,
      capacity: Self::CAPACITY,
      mark_room: 0,
      period: queue.get_timestamp_period(),
      is_enabled: false,
      frame: None,
      sums: Vec::new(),
      frames: Vec::new(),
      tag: 0,
      tagged: Vec::new(),
      zones: GpuZones::default(),
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
      self.frames.clear();
      self.tagged.clear();
    }
  }

  /// Tags the frames timed from now on, which `take_tagged` answers their passes' times under.
  pub fn set_tag(&mut self, tag: u64) {
    self.tag = tag;
  }

  /// Every pass of every frame read back since the last call, each under the tag its frame was timed with, in the order
  /// they were read back.
  pub fn take_tagged(&mut self) -> Vec<GraphTaggedTime> {
    self.collect();

    std::mem::take(&mut self.tagged)
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

  /// Each of an owner's passes' mean GPU milliseconds a frame over the frames read back since that owner's last call,
  /// which starts its next span: a pass a frame did not run, as a cascade redrawn only when due, counts nought there, so
  /// the passes add up to the frame. `FrameGraph::FRAME_OWNER` for passes no owner was named for.
  pub fn take(&mut self, owner: u32) -> Vec<GraphPassTime> {
    self.collect();

    let (taken, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.sums)
      .into_iter()
      .partition(|(it, ..)| *it == owner);
    let frames: u32 = self
      .frames
      .iter()
      .find(|(it, _)| *it == owner)
      .map_or(0, |(_, frames)| *frames);

    self.sums = kept;
    self.frames.retain(|(it, _)| *it != owner);

    taken
      .into_iter()
      .map(|(_, name, total)| GraphPassTime {
        name,
        gpu_time: (total / f64::from(frames.max(1))) as f32,
      })
      .collect()
  }

  /// Starts timing a frame of groups writing `counts` stamps each, unless timing is off or every slot is still on its
  /// way back: answers each group's range, the last resolving the frame. A frame past the capacity grows the timer
  /// once every slot is back, and goes untimed until then.
  pub(crate) fn begin_frame(&mut self, device: &wgpu::Device, counts: &[u32]) -> Option<Vec<GroupStamps>> {
    self.collect();
    self.frame = None;

    if !self.is_timing() || counts.is_empty() {
      return None;
    }

    let room: u32 = self.mark_room;
    let ranges: Vec<(u32, u32)> = counts
      .iter()
      .scan(0, |start: &mut u32, count| {
        let range: (u32, u32) = (*start, count + room);

        *start += range.1;

        Some(range)
      })
      .collect();
    let total: u32 = ranges.last().map_or(0, |(start, count)| start + count);

    if total > self.capacity {
      if !self.is_idle() {
        return None;
      }

      let capacity: u32 = total.next_power_of_two();

      (self.queries, self.resolve, self.slots) = Self::create(device, capacity);
      self.capacity = capacity;
    }

    let index: usize = self
      .slots
      .iter()
      .position(|slot| slot.state.load(Ordering::Acquire) == TimerSlot::IDLE)?;
    let queries: wgpu::QuerySet = self.queries.clone()?;
    let last: usize = ranges.len() - 1;
    let stamps: Vec<GroupStamps> = ranges
      .iter()
      .enumerate()
      .map(|(group, range)| {
        let resolve = (group == last).then(|| (self.resolve.clone(), self.slots[index].buffer.clone(), total));

        GroupStamps::new(queries.clone(), *range, resolve)
      })
      .collect();

    self.frame = Some((index, ranges));

    Some(stamps)
  }

  /// Takes a recorded frame's stamps into its slot, asked for once the frame is submitted. A frame a group of which
  /// asked for more stamps than its range goes untimed, and later frames give every group that much more room.
  pub(crate) fn finish_frame(&mut self, groups: Vec<GroupStamps>) {
    let Some((index, ranges)) = self.frame.take() else {
      return;
    };
    let overflow: u32 = groups.iter().map(GroupStamps::get_overflow).max().unwrap_or(0);

    if overflow > 0 {
      self.mark_room += overflow;

      return;
    }

    let total: u32 = ranges.last().map_or(0, |(start, count)| start + count);
    let mut names: Vec<Option<TimedPass>> = vec![None; total as usize];

    for ((start, _), group) in ranges.iter().zip(groups) {
      for (offset, name) in group.names.into_iter().enumerate() {
        names[*start as usize + offset] = name;
      }
    }

    let slot: &mut TimerSlot = &mut self.slots[index];

    slot.names = names;
    slot.tag = self.tag;
    slot.state.store(TimerSlot::RECORDED, Ordering::Release);
  }

  fn is_idle(&self) -> bool {
    self
      .slots
      .iter()
      .all(|slot| slot.state.load(Ordering::Acquire) == TimerSlot::IDLE)
  }

  /// The query set, the resolve buffer and the readback slots for frames of up to `capacity` stamps.
  fn create(device: &wgpu::Device, capacity: u32) -> (Option<wgpu::QuerySet>, wgpu::Buffer, Vec<TimerSlot>) {
    let size: u64 = u64::from(capacity) * 8;
    let queries: Option<wgpu::QuerySet> = device
      .features()
      .contains(wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
      .then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
          label: Some("graph timer"),
          ty: wgpu::QueryType::Timestamp,
          count: capacity,
        })
      });
    let resolve: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("graph timer resolve"),
      size,
      usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
      mapped_at_creation: false,
    });
    let slots: Vec<TimerSlot> = (0..Self::SLOTS)
      .map(|_| TimerSlot {
        buffer: device.create_buffer(&wgpu::BufferDescriptor {
          label: Some("graph timer readback"),
          size,
          usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
          mapped_at_creation: false,
        }),
        state: Arc::new(AtomicU8::new(TimerSlot::IDLE)),
        names: Vec::new(),
        tag: 0,
      })
      .collect();

    (queries, resolve, slots)
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

        self.zones.send(&stamps, &slot.names, self.period);

        let mut owners: Vec<u32> = Vec::new();

        for (index, name) in slot.names.iter().enumerate() {
          let Some((owner, name)) = name.as_ref().filter(|_| index > 0) else {
            continue;
          };
          // A tick count running backwards across a pass is a driver's, not a negative cost.
          let spent: f64 = stamps[index].saturating_sub(stamps[index - 1]) as f64 * to_milliseconds;

          match self.sums.iter_mut().find(|(it, named, _)| it == owner && named == name) {
            Some((_, _, total)) => *total += spent,
            None => self.sums.push((*owner, name.clone(), spent)),
          }

          self.tagged.push(GraphTaggedTime {
            tag: slot.tag,
            name: name.clone(),
            gpu_time: spent as f32,
          });

          if !owners.contains(owner) {
            owners.push(*owner);
          }
        }

        // Each owner's frame counted once, whatever it ran.
        for owner in owners {
          match self.frames.iter_mut().find(|(it, _)| *it == owner) {
            Some((_, frames)) => *frames += 1,
            None => self.frames.push((owner, 1)),
          }
        }
      }

      slot.buffer.unmap();
      slot.state.store(TimerSlot::IDLE, Ordering::Release);
    }

    let excess: usize = self.tagged.len().saturating_sub(Self::TAGGED_MOST);

    self.tagged.drain(..excess);
  }
}
