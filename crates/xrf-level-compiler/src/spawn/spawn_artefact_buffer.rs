/// The vector xrAI floods each anomalous zone's nodes into (`l_tpaStack`, `level_spawn_constructor.cpp:442`), kept as
/// its memory holds them: reused from zone to zone, cleared without its elements going, and read past its end where a
/// zone wants more points than its flood kept, which gives the previous zone's nodes, as the shipped spawn holds them.
pub(crate) struct SpawnArtefactBuffer {
  /// The vector's memory, its capacity long; past what any zone wrote, nothing the engine would read the same.
  memory: Vec<u32>,
}

impl SpawnArtefactBuffer {
  /// `l_tpaStack.reserve(1024)`.
  const RESERVED: usize = 1024;
  /// What memory past every write reads as.
  const UNWRITTEN: u32 = 0;

  /// `push_back` of each flooded node into the cleared vector, then `remove_if` of the nodes `keep` refuses and
  /// `erase`: the kept ones packed to the front in order, the memory after them as the pushes left it. Returns how
  /// many it kept.
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

  /// The first `count` elements as the memory holds them, past the vector's end and its capacity too.
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
