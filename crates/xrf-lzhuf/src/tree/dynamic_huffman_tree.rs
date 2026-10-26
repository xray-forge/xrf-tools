use xrf_error::{XrfError, XrfResult};

use crate::bits::{BitReader, BitWriter};
use crate::lzhuf_constants::{LEAF_COUNT, NODE_COUNT, REBUILD_FREQUENCY_LIMIT};
use crate::tree::{NodeType, TreeEntry};

/// One node of the adaptive tree.
#[derive(Clone, Copy)]
struct TreeNode {
  entry: TreeEntry,
  frequency: u16,
  /// Index of this node's parent; the root parents itself.
  parent: u16,
  /// Identifier shared by the run of adjacent nodes with the same frequency.
  group: u16,
}

impl Default for TreeNode {
  fn default() -> Self {
    Self {
      entry: TreeEntry::branch(0),
      frequency: 0,
      parent: 0,
      group: 0,
    }
  }
}

impl TreeNode {
  fn new_leaf(value: u16, group: u16) -> Self {
    Self {
      entry: TreeEntry::leaf(value),
      frequency: 1,
      parent: 0,
      group,
    }
  }

  fn new_branch(child_index: u16, frequency: u16, group: u16) -> Self {
    Self {
      entry: TreeEntry::branch(child_index),
      frequency,
      parent: 0,
      group,
    }
  }
}

/// Where each coded value currently lives in the node array.
#[derive(Clone)]
struct LeafIndex([u16; LEAF_COUNT]);

impl LeafIndex {
  fn set(&mut self, value: u16, node_index: usize) {
    debug_assert!(node_index < NODE_COUNT);

    self.0[value as usize] = node_index as u16;
  }

  fn node_index_of(&self, value: u16) -> usize {
    self.0[value as usize] as usize
  }
}

/// Identifier pool for the equal-frequency runs the update rule promotes nodes within.
///
/// Every array access below is bounded by construction: there is never more than one group per node, and
/// group identifiers only ever come out of this pool. Values decoded from a stream cannot widen either
/// bound, because they are leaf values the tree itself produced.
#[derive(Clone)]
struct FrequencyGroups {
  allocated: u16,
  /// Free-list of identifiers, consumed from the front.
  pool: [u16; NODE_COUNT],
  /// First node index belonging to each group.
  leaders: [u16; NODE_COUNT],
}

impl FrequencyGroups {
  fn new() -> Self {
    let mut groups: Self = Self {
      allocated: 0,
      pool: [0; NODE_COUNT],
      // The root's group is allocated first and is never assigned a leader explicitly during
      // construction, so zero here is what puts the root's leader at node 0.
      leaders: [0; NODE_COUNT],
    };

    groups.reset();

    groups
  }

  fn reset(&mut self) {
    self.allocated = 0;

    for (slot, identifier) in self.pool.iter_mut().zip(0u16..) {
      *slot = identifier;
    }
  }

  fn allocate(&mut self) -> u16 {
    let identifier: u16 = self.pool[self.allocated as usize];

    self.allocated += 1;

    identifier
  }

  fn free(&mut self, group: u16) {
    self.allocated -= 1;
    self.pool[self.allocated as usize] = group;
  }

  fn set_leader(&mut self, group: u16, node_index: usize) {
    debug_assert!(node_index < NODE_COUNT);

    self.leaders[group as usize] = node_index as u16;
  }

  fn leader_of(&self, group: u16) -> usize {
    self.leaders[group as usize] as usize
  }

  /// Hand leadership of a group to the node after its current leader.
  fn advance_leader(&mut self, group: u16) {
    self.leaders[group as usize] += 1;
  }
}

/// The adaptive Huffman tree both sides of the codec keep in lockstep.
///
/// Nodes are held in one array ordered by descending frequency, so a node is promoted by swapping it with
/// the leader of its frequency group rather than by relinking pointers. Decoding a value and coding one
/// both end in the same frequency update, which is what keeps the encoder's tree and the decoder's tree
/// identical without either side transmitting the tree.
#[derive(Clone)]
pub(crate) struct DynamicHuffmanTree {
  nodes: [TreeNode; NODE_COUNT],
  leaf_index: LeafIndex,
  groups: FrequencyGroups,
}

impl DynamicHuffmanTree {
  /// Build the balanced starting tree, where every value has frequency 1.
  ///
  /// Both sides start from this exact shape; it is never transmitted.
  pub(crate) fn new() -> Self {
    let mut groups: FrequencyGroups = FrequencyGroups::new();
    let mut nodes: [TreeNode; NODE_COUNT] = [TreeNode::default(); NODE_COUNT];
    let leaf_index: LeafIndex = LeafIndex(std::array::from_fn(|value| (NODE_COUNT - 1 - value) as u16));

    let mut last_group: u16 = groups.allocate();

    // Leaves occupy the tail of the array in descending value order.
    for (node, value) in nodes[NODE_COUNT - LEAF_COUNT..].iter_mut().rev().zip(0u16..) {
      *node = TreeNode::new_leaf(value, last_group);
    }

    // Fold the tail pairwise into parents until only the root is left.
    let mut tail_length: usize = LEAF_COUNT;
    let mut rest: &mut [TreeNode] = &mut nodes[..];
    let mut last_frequency: u16 = 0;

    while tail_length > 1 {
      let rest_length: usize = rest.len();
      let parent_length: usize = tail_length / 2;
      let (head, children) = rest.split_at_mut(rest_length - parent_length * 2);
      let head_end: usize = head.len() - (tail_length & 1);

      for ((child_nodes, child_index), (index, parent_node)) in children
        .rchunks_exact_mut(2)
        .zip((0..rest_length).rev().step_by(2))
        .zip(head[..head_end].iter_mut().enumerate().rev())
      {
        let mut frequency: u16 = 0;

        for child in child_nodes.iter_mut() {
          frequency += child.frequency;
          child.parent = index as u16;
        }

        if frequency != last_frequency {
          groups.set_leader(last_group, index + 1);
          last_frequency = frequency;
          last_group = groups.allocate();
        }

        *parent_node = TreeNode::new_branch(child_index as u16, frequency, last_group);
      }

      tail_length -= parent_length;
      rest = head;
    }

    Self {
      nodes,
      leaf_index,
      groups,
    }
  }

  /// Walk from the root to a leaf, one stream bit per branch, and account for the value read.
  pub(crate) fn read_code(&mut self, reader: &mut BitReader) -> XrfResult<u16> {
    let mut index: usize = 0;

    loop {
      let Some(node) = self.nodes.get(index) else {
        return Err(XrfError::new_parsing_error(format!(
          "LZHUF tree walk left the node array at index {index}"
        )));
      };

      match node.entry.as_type() {
        NodeType::Leaf(value) => {
          self.increment_frequency_for(value)?;

          return Ok(value);
        }
        NodeType::Branch(child_index) => {
          // A branch names its higher child, and a zero bit steps to the lower one.
          let bit: usize = usize::from(reader.read_bits(1)?);

          index = (child_index as usize).checked_sub(bit).ok_or_else(|| {
            XrfError::new_parsing_error(format!("LZHUF tree branch {child_index} has no lower child"))
          })?;
        }
      }
    }
  }

  /// Write the path from the root down to `value`'s leaf, then account for the value written.
  ///
  /// The exact mirror of [`Self::read_code`]: the walk collects bits from the leaf upward, so they are
  /// emitted in reverse to put the root's bit first, and the frequency update happens after coding on
  /// both sides. That ordering is what keeps the two trees identical step for step.
  pub(crate) fn write_code(&mut self, value: u16, writer: &mut BitWriter) -> XrfResult<()> {
    let mut node_index: usize = self.leaf_index.node_index_of(value);
    let mut code: u32 = 0;
    let mut length: u32 = 0;

    while node_index != 0 {
      let parent_index: usize = self.node(node_index)?.parent as usize;

      let NodeType::Branch(child_index) = self.node(parent_index)?.entry.as_type() else {
        return Err(XrfError::new_encoding_error(format!(
          "LZHUF tree node {parent_index} parents node {node_index} without being a branch"
        )));
      };

      // Reading a bit steps from the named child down by that bit, so this node's bit is the distance.
      let bit: u32 = match usize::from(child_index).checked_sub(node_index) {
        Some(bit @ (0 | 1)) => bit as u32,
        _ => {
          return Err(XrfError::new_encoding_error(format!(
            "LZHUF tree node {node_index} is not a child of branch {child_index}"
          )));
        }
      };

      code |= bit << length;
      length += 1;
      node_index = parent_index;
    }

    writer.write_bits(code, length)?;

    self.increment_frequency_for(value)
  }

  /// Account for one occurrence of `value`, re-ordering the tree so frequencies stay descending.
  fn increment_frequency_for(&mut self, value: u16) -> XrfResult<()> {
    if self.nodes[0].frequency >= REBUILD_FREQUENCY_LIMIT {
      self.rebuild()?;
    }

    self.nodes[0].frequency += 1;

    let mut node_index: usize = self.leaf_index.node_index_of(value);

    while node_index != 0 {
      node_index = self.promote_to_leader(node_index)?;
      node_index = self.bump_frequency(node_index)?;
    }

    Ok(())
  }

  /// Swap a node with the first node of its frequency group, so incrementing it keeps the array ordered.
  fn promote_to_leader(&mut self, node_index: usize) -> XrfResult<usize> {
    let group: u16 = self.node(node_index)?.group;
    let leader_index: usize = self.groups.leader_of(group);

    if leader_index == node_index {
      return Ok(node_index);
    }

    if leader_index > node_index {
      return Err(XrfError::new_parsing_error(format!(
        "LZHUF group leader {leader_index} follows its member {node_index}"
      )));
    }

    let node_entry: TreeEntry = self.node(node_index)?.entry;
    let leader_entry: TreeEntry = self.node(leader_index)?.entry;

    self.nodes[node_index].entry = leader_entry;
    self.nodes[leader_index].entry = node_entry;

    self.relink(leader_entry, node_index)?;
    self.relink(node_entry, leader_index)?;

    Ok(leader_index)
  }

  /// Point whatever an entry owns - a coded value or a pair of children - at the entry's new node index.
  fn relink(&mut self, entry: TreeEntry, node_index: usize) -> XrfResult<()> {
    match entry.as_type() {
      NodeType::Leaf(value) => self.leaf_index.set(value, node_index),
      NodeType::Branch(child_index) => {
        let higher: usize = child_index as usize;
        let lower: usize = higher
          .checked_sub(1)
          .ok_or_else(|| XrfError::new_parsing_error(format!("LZHUF tree branch {child_index} has no lower child")))?;

        for child in [lower, higher] {
          let Some(node) = self.nodes.get_mut(child) else {
            return Err(XrfError::new_parsing_error(format!(
              "LZHUF tree child {child} is outside the node array"
            )));
          };

          node.parent = node_index as u16;
        }
      }
    }

    Ok(())
  }

  /// Add one to a node's frequency, moving it between frequency groups as needed, and return its parent.
  fn bump_frequency(&mut self, node_index: usize) -> XrfResult<usize> {
    let previous: &TreeNode = self.node(node_index.checked_sub(1).ok_or_else(|| {
      XrfError::new_parsing_error("LZHUF frequency update reached above the tree root".to_string())
    })?)?;

    let previous_frequency: u16 = previous.frequency;
    let previous_group: u16 = previous.group;

    let node: &mut TreeNode = self.node_mut(node_index)?;

    node.frequency += 1;

    let frequency: u16 = node.frequency;
    let group: u16 = node.group;
    let parent: usize = node.parent as usize;

    // The node still shares its group with the nodes after it, so the next one takes over the group.
    if self.nodes.get(node_index + 1).is_some_and(|next| next.group == group) {
      self.groups.advance_leader(group);

      if frequency == previous_frequency {
        self.nodes[node_index].group = previous_group;
      } else {
        let allocated: u16 = self.groups.allocate();

        self.nodes[node_index].group = allocated;
        self.groups.set_leader(allocated, node_index);
      }

      return Ok(parent);
    }

    // The node was alone in its group, so that group is spent once the node joins the previous one.
    if frequency == previous_frequency {
      self.groups.free(group);
      self.nodes[node_index].group = previous_group;
    }

    Ok(parent)
  }

  /// Halve every frequency and rebuild the tree bottom-up once the root frequency saturates.
  ///
  /// Both sides rebuild at the same point from the same state, so the rebuilt trees match. Frequencies
  /// round up while halving, which keeps every leaf non-zero and so keeps every value codable.
  fn rebuild(&mut self) -> XrfResult<()> {
    let mut leaves: [(TreeEntry, u16); LEAF_COUNT] = [(TreeEntry::leaf(0), 0); LEAF_COUNT];
    let mut taken: usize = 0;

    // Leaves keep their current relative order, which the leaf index cannot supply.
    for node in self.nodes.iter().filter(|node| node.entry.is_leaf()) {
      let Some(slot) = leaves.get_mut(taken) else {
        return Err(XrfError::new_parsing_error(
          "LZHUF tree holds more leaves than it has values".to_string(),
        ));
      };

      *slot = (node.entry, node.frequency.div_ceil(2));
      taken += 1;
    }

    if taken != LEAF_COUNT {
      return Err(XrfError::new_parsing_error(format!(
        "LZHUF tree holds {taken} leaves, expected {LEAF_COUNT}"
      )));
    }

    let mut remaining_leaves = leaves.into_iter().rev();
    let mut next_leaf: Option<(TreeEntry, u16)> = remaining_leaves.next();
    let mut target_length: usize = NODE_COUNT;
    let mut nodes: &mut [TreeNode] = &mut self.nodes[..];

    while nodes.len() > 2 {
      let child_index: usize = nodes.len() - 1;
      let child_count: usize = child_index + 1 - target_length;

      // A branch needs two children, so pull down the least frequent leaves until it has them.
      if child_count < 2 {
        for node in nodes[..target_length].iter_mut().rev().take(2 - child_count) {
          let Some((entry, frequency)) = next_leaf else {
            return Err(XrfError::new_parsing_error(
              "LZHUF tree rebuild ran out of leaves while filling children".to_string(),
            ));
          };

          target_length -= 1;
          self.leaf_index.set(entry.as_value(), target_length);
          node.entry = entry;
          node.frequency = frequency;
          next_leaf = remaining_leaves.next();
        }
      }

      let (head, children) = nodes.split_at_mut(target_length);
      let branch_frequency: u16 = children.iter().rev().take(2).map(|node| node.frequency).sum();
      let mut target = head.iter_mut().rev();

      // Any leaf at least as frequent as the new branch sorts ahead of it.
      while let Some((entry, frequency)) = next_leaf {
        if branch_frequency < frequency {
          break;
        }

        let Some(node) = target.next() else {
          return Err(XrfError::new_parsing_error(
            "LZHUF tree rebuild ran out of room for leaves".to_string(),
          ));
        };

        self.leaf_index.set(entry.as_value(), target.len());
        node.entry = entry;
        node.frequency = frequency;
        next_leaf = remaining_leaves.next();
      }

      let Some(node) = target.next() else {
        return Err(XrfError::new_parsing_error(
          "LZHUF tree rebuild ran out of room for branches".to_string(),
        ));
      };

      node.entry.set_as_branch(child_index as u16);
      node.frequency = branch_frequency;
      target_length = target.len();

      for child in children.iter_mut().rev().take(2) {
        child.parent = target_length as u16;
      }

      nodes = &mut nodes[..child_index - 1];
    }

    self.rebuild_groups();

    Ok(())
  }

  /// Re-derive the equal-frequency runs after a rebuild has reordered every node.
  fn rebuild_groups(&mut self) {
    self.groups.reset();

    let mut group: u16 = self.groups.allocate();
    let mut frequency: u16 = self.nodes[0].frequency;

    self.nodes[0].group = group;
    self.groups.set_leader(group, 0);

    for (offset, node) in self.nodes[1..].iter_mut().enumerate() {
      if node.frequency == frequency {
        node.group = group;
      } else {
        frequency = node.frequency;
        group = self.groups.allocate();
        node.group = group;
        self.groups.set_leader(group, offset + 1);
      }
    }
  }

  fn node(&self, index: usize) -> XrfResult<&TreeNode> {
    self
      .nodes
      .get(index)
      .ok_or_else(|| XrfError::new_parsing_error(format!("LZHUF tree node {index} is outside the node array")))
  }

  fn node_mut(&mut self, index: usize) -> XrfResult<&mut TreeNode> {
    self
      .nodes
      .get_mut(index)
      .ok_or_else(|| XrfError::new_parsing_error(format!("LZHUF tree node {index} is outside the node array")))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::{HashMap, HashSet};

  use super::{DynamicHuffmanTree, TreeNode};
  use crate::bits::BitReader;
  use crate::lzhuf_constants::{LEAF_COUNT, NODE_COUNT, REBUILD_FREQUENCY_LIMIT};
  use crate::tree::NodeType;

  /// Assert every invariant the decode walk and the update rule rely on.
  fn assert_tree_is_sound(tree: &DynamicHuffmanTree) {
    let mut leaves: HashMap<u16, usize> = HashMap::with_capacity(LEAF_COUNT);
    let mut children: HashSet<u16> = HashSet::with_capacity(NODE_COUNT);
    let mut groups: HashSet<u16> = HashSet::with_capacity(NODE_COUNT);
    let mut frequency: u16 = u16::MAX;
    let mut group: u16 = u16::MAX;

    assert!(!tree.nodes[0].entry.is_leaf(), "root stays a branch");

    for (index, node) in tree.nodes.iter().enumerate() {
      match node.entry.as_type() {
        NodeType::Leaf(value) => {
          assert!((value as usize) < LEAF_COUNT, "leaf value in range");
          assert!(leaves.insert(value, index).is_none(), "leaf values are unique");
        }
        NodeType::Branch(child_index) => {
          assert_ne!(child_index, 0, "no default node survives");
          assert_eq!(child_index & 1, 0, "children are paired on even indices");
          assert!(children.insert(child_index), "children have one parent");

          let lower: &TreeNode = &tree.nodes[child_index as usize - 1];
          let higher: &TreeNode = &tree.nodes[child_index as usize];

          assert_eq!(lower.parent as usize, index);
          assert_eq!(higher.parent as usize, index);
          assert_eq!(lower.frequency + higher.frequency, node.frequency);
        }
      }

      assert!(node.frequency <= frequency, "frequencies descend");

      if node.frequency == frequency {
        assert_eq!(node.group, group, "equal frequencies share a group");
      } else {
        assert_ne!(node.group, group, "a frequency change starts a group");
        assert!(groups.insert(node.group), "groups are unique");
        assert_eq!(
          tree.groups.leader_of(node.group),
          index,
          "group leads at its first node"
        );

        group = node.group;
        frequency = node.frequency;
      }
    }

    assert_eq!(leaves.len(), LEAF_COUNT, "every value stays codable");

    for (value, index) in leaves {
      assert_eq!(tree.leaf_index.node_index_of(value), index, "leaf index agrees");
    }
  }

  #[test]
  fn starts_sound_and_balanced() {
    assert_tree_is_sound(&DynamicHuffmanTree::new());
  }

  #[test]
  fn stays_sound_while_frequencies_skew() {
    let mut tree: DynamicHuffmanTree = DynamicHuffmanTree::new();

    for value in 0..LEAF_COUNT as u16 {
      for _ in 0..value {
        tree.increment_frequency_for(value).expect("frequency update");
      }
    }

    assert_tree_is_sound(&tree);
  }

  #[test]
  fn stays_sound_across_a_rebuild() {
    let mut tree: DynamicHuffmanTree = DynamicHuffmanTree::new();

    // Push the root past the rebuild limit, which is the X-Ray specific behaviour worth pinning.
    for _ in 0..REBUILD_FREQUENCY_LIMIT as usize * 2 {
      tree.increment_frequency_for(7).expect("frequency update");
    }

    assert!(
      tree.nodes[0].frequency < REBUILD_FREQUENCY_LIMIT,
      "rebuild reset the root"
    );
    assert_tree_is_sound(&tree);
  }

  /// Deterministic bits, so the golden test below needs no committed fixture.
  fn pseudo_random_source(length: usize) -> Vec<u8> {
    let mut state: u32 = 0x1234_5678;

    (0..length)
      .map(|_| {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);

        (state >> 24) as u8
      })
      .collect()
  }

  /// Pins the decoded symbol sequence across a rebuild, which is where X-Ray leaves standard LHA behind.
  ///
  /// Any bit pattern is a valid walk - every path ends at a leaf - so an arbitrary stream exercises the
  /// coder without needing an encoder. The digest covers more symbols than [`REBUILD_FREQUENCY_LIMIT`],
  /// so it only holds while the tree rebuilds at X-Ray's halved limit: raising the limit to the standard
  /// 32768, or disturbing the rebuild, changes it. Real archives proved this exact code path byte for
  /// byte against the previously vendored decoder before that decoder was removed.
  #[test]
  fn decodes_a_known_symbol_sequence_across_a_rebuild() {
    const CODE_COUNT: usize = 20_000;

    let source: Vec<u8> = pseudo_random_source(64 * 1024);
    let mut reader: BitReader = BitReader::new(&source);
    let mut tree: DynamicHuffmanTree = DynamicHuffmanTree::new();
    let mut digest: u64 = 0xcbf2_9ce4_8422_2325;

    assert!(
      CODE_COUNT > REBUILD_FREQUENCY_LIMIT as usize,
      "sequence spans a rebuild"
    );

    for _ in 0..CODE_COUNT {
      let value: u16 = tree.read_code(&mut reader).expect("stream long enough for every code");

      assert!((value as usize) < LEAF_COUNT);

      digest = (digest ^ u64::from(value)).wrapping_mul(0x0000_0100_0000_01b3);
    }

    assert_eq!(digest, 0xf750_9dc2_6ba1_ed02);
    assert_tree_is_sound(&tree);
  }
}
