use std::ops::Range;

/// Writes waiting for the next upload, by byte offset; coalesced into contiguous runs, so a scene's many small changes
/// go up as few writes.
#[derive(Default)]
pub struct SpanWrites {
  entries: Vec<(u64, Range<usize>)>,
  data: Vec<u8>,
}

impl SpanWrites {
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Queues `bytes` for the buffer at `offset` bytes; a later write to the same bytes wins.
  pub fn push(&mut self, offset: u64, bytes: &[u8]) {
    let start: usize = self.data.len();

    self.data.extend_from_slice(bytes);
    self.entries.push((offset, start..self.data.len()));
  }

  /// The writes as runs of contiguous bytes, in offset order, emptying them; a later write to the same bytes wins.
  pub fn take_runs(&mut self) -> Vec<(u64, Vec<u8>)> {
    let entries: Vec<(u64, Range<usize>)> = std::mem::take(&mut self.entries);
    let data: Vec<u8> = std::mem::take(&mut self.data);
    // The runs' extents, from the writes' in offset order, each write joining the run it touches.
    let mut spans: Vec<(u64, u64)> = entries
      .iter()
      .map(|(offset, range)| (*offset, *offset + range.len() as u64))
      .collect();

    spans.sort_unstable();

    let mut extents: Vec<(u64, u64)> = Vec::new();

    for (start, end) in spans {
      match extents.last_mut() {
        Some((_, last)) if start <= *last => *last = (*last).max(end),
        _ => extents.push((start, end)),
      }
    }

    let mut runs: Vec<(u64, Vec<u8>)> = extents
      .iter()
      .map(|(start, end)| (*start, vec![0; (end - start) as usize]))
      .collect();

    // Each write laid over its run in the order it was pushed, so a later one wins wherever two overlap, whichever
    // starts first.
    for (offset, range) in entries {
      let (start, run) = &mut runs[extents.partition_point(|(start, _)| *start <= offset) - 1];
      let at: usize = (offset - *start) as usize;

      run[at..at + range.len()].copy_from_slice(&data[range]);
    }

    runs
  }
}
