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

  pub fn push(&mut self, offset: u64, bytes: &[u8]) {
    let start: usize = self.data.len();

    self.data.extend_from_slice(bytes);
    self.entries.push((offset, start..self.data.len()));
  }

  /// The writes as runs of contiguous bytes, in offset order, emptying them; a later write to the same bytes wins.
  pub fn take_runs(&mut self) -> Vec<(u64, Vec<u8>)> {
    let mut entries: Vec<(usize, (u64, Range<usize>))> =
      std::mem::take(&mut self.entries).into_iter().enumerate().collect();
    let data: Vec<u8> = std::mem::take(&mut self.data);

    // Stable by offset and then by order pushed, so an overlapping later write is applied over an earlier one.
    entries.sort_by_key(|(order, (offset, _))| (*offset, *order));

    let mut runs: Vec<(u64, Vec<u8>)> = Vec::new();

    for (_, (offset, range)) in entries {
      let bytes: &[u8] = &data[range];

      match runs.last_mut() {
        Some((start, run)) if offset <= *start + run.len() as u64 => {
          let at: usize = (offset - *start) as usize;
          let end: usize = at + bytes.len();

          if end > run.len() {
            run.resize(end, 0);
          }

          run[at..end].copy_from_slice(bytes);
        }
        _ => runs.push((offset, bytes.to_vec())),
      }
    }

    runs
  }
}
