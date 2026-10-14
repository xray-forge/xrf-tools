use crate::graph::timing::timed_pass::TimedPass;

/// One encode group's share of a timed frame's stamps: a range of the timer's query set of its own, so the groups
/// record their stamps at once on threads of their own; the last group also resolves the frame's stamps into its slot.
pub(crate) struct GroupStamps {
  queries: wgpu::QuerySet,
  /// The group's first query and how many it has.
  start: u32,
  capacity: u32,
  /// The pass each stamp ends, by stamp from `start`; `None` for the stamp starting the group. Past the capacity it
  /// still counts, and the frame goes untimed.
  pub names: Vec<Option<TimedPass>>,
  /// The resolve buffer, the slot's buffer and the frame's stamps, for the last group.
  resolve: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
}

impl GroupStamps {
  pub fn new(
    queries: wgpu::QuerySet,
    (start, capacity): (u32, u32),
    resolve: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
  ) -> Self {
    Self {
      queries,
      start,
      capacity,
      names: Vec::with_capacity(capacity as usize),
      resolve,
    }
  }

  /// Writes a stamp ending an owner's pass by its name, or starting the group where `None`.
  pub fn stamp(&mut self, encoder: &mut wgpu::CommandEncoder, name: Option<TimedPass>) {
    let index: u32 = self.names.len() as u32;

    if index < self.capacity {
      encoder.write_timestamp(&self.queries, self.start + index);
    }

    self.names.push(name);
  }

  /// Resolves the frame's stamps into its slot where this is the last group, after its own: the groups' command
  /// buffers are submitted in order, so every other group's stamps are written by then.
  pub fn finish(&self, encoder: &mut wgpu::CommandEncoder) {
    if let Some((resolve, slot, count)) = &self.resolve {
      encoder.resolve_query_set(&self.queries, 0..*count, resolve, 0);
      encoder.copy_buffer_to_buffer(resolve, 0, slot, 0, u64::from(*count) * 8);
    }
  }

  /// Stamps it asked for past its range.
  pub fn get_overflow(&self) -> u32 {
    (self.names.len() as u32).saturating_sub(self.capacity)
  }
}
