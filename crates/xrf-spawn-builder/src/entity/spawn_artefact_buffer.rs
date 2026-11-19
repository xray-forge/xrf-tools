/// The vector xrAI floods each anomalous zone's nodes into (`l_tpaStack`), modelled as its memory.
///
/// xrAI reuses it across zones without erasing elements and reads past its end where a zone wants more points than its
/// flood kept, so such a zone gets the previous zone's nodes, as the shipped spawn holds them.
pub(crate) struct SpawnArtefactBuffer {
  /// The vector's memory, its capacity long.
  memory: Vec<u32>,
}

impl SpawnArtefactBuffer {
  /// `l_tpaStack.reserve(1024)`.
  const RESERVED: usize = 1024;
  /// What memory past every write reads as; the engine's would read whatever it held, so it cannot match there.
  const UNWRITTEN: u32 = 0;

  /// Pushes `flooded` into the cleared vector, then `remove_if` and `erase` of the nodes `keep` refuses, returning how
  /// many it kept.
  ///
  /// The kept nodes are packed to the front in order; the memory after them stays as the pushes left it.
  pub(crate) fn fill(&mut self, flooded: &[u32], keep: impl Fn(u32) -> bool) -> usize {
    let mut capacity: usize = self.memory.len();

    while capacity < flooded.len() {
      // VS2008's `vector` grows by half again when full; what lay past the old size is not carried over.
      capacity = (capacity + capacity / 2).max(capacity + 1);
      self.memory.clear();
    }

    self.memory.resize(capacity, Self::UNWRITTEN);
    self.memory[..flooded.len()].copy_from_slice(flooded);

    let mut kept: usize = 0;

    for index in 0..flooded.len() {
      let node: u32 = self.memory[index];

      if keep(node) {
        self.memory[kept] = node;
        kept += 1;
      }
    }

    kept
  }

  /// The first `count` elements, to shuffle.
  pub(crate) fn get_mut(&mut self, count: usize) -> &mut [u32] {
    &mut self.memory[..count]
  }

  /// The first `count` elements as the memory holds them, even past the vector's end and capacity.
  pub(crate) fn read(&mut self, count: usize) -> &[u32] {
    if self.memory.len() < count {
      self.memory.resize(count, Self::UNWRITTEN);
    }

    &self.memory[..count]
  }
}

impl Default for SpawnArtefactBuffer {
  fn default() -> Self {
    Self {
      memory: vec![Self::UNWRITTEN; Self::RESERVED],
    }
  }
}
