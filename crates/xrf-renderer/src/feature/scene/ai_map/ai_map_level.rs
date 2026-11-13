use glam::{UVec4, Vec4};
use wgpu::util::DeviceExt;

use crate::feature::scene::ai_map::ai_map_field::AiMapField;
use crate::feature::scene::ai_map::ai_map_uniform::AiMapUniform;
use crate::host::level::ai_map::render_level_ai_map::RenderLevelAiMap;

/// A level's AI map as its debug views read it: a grid of cells, each naming the run of nodes standing on it, and each
/// node's height, covers and missing links.
pub(crate) struct AiMapLevel {
  /// One word a cell, x in rows of the cells along z: the first node standing on it shifted up by eight, the nodes
  /// standing on it in the low eight bits, none there where zero. The map keeps a cell's nodes side by side.
  pub cells: wgpu::Buffer,
  /// Two words a node: its packed height with its missing links' bits above it, and its high covers with its low ones
  /// above them.
  pub nodes: wgpu::Buffer,
  origin: Vec4,
  row_length: u32,
  column_length: u32,
  node_count: u32,
  height: f32,
}

impl AiMapLevel {
  /// Cells the grid may hold, past which a map is not shown: four bytes a cell, a quarter of a gigabyte.
  const MAX_CELLS: u64 = 1 << 26;

  /// Nodes a cell's word can name the first of.
  const MAX_NODES: usize = 1 << 24;

  /// The map's buffers, none for a map whose grid or nodes are past what the words hold.
  pub(crate) fn new(device: &wgpu::Device, map: &RenderLevelAiMap) -> Option<Self> {
    let cell_count: u64 = u64::from(map.row_length) * u64::from(map.column_length);

    if cell_count == 0 || cell_count > Self::MAX_CELLS || map.nodes.len() >= Self::MAX_NODES {
      log::warn!(
        "Native viewport AI map of {} nodes over {cell_count} cells is not shown",
        map.nodes.len()
      );

      return None;
    }

    let mut cells: Vec<u32> = vec![0; cell_count as usize];
    let mut nodes: Vec<u32> = Vec::with_capacity(map.nodes.len() * 2);

    for (index, node) in map.nodes.iter().enumerate() {
      if let Some(cell) = cells.get_mut(node.cell as usize) {
        *cell = if *cell == 0 {
          ((index as u32) << 8) | 1
        } else {
          *cell + u32::from(*cell & 0xFF != 0xFF)
        };
      }

      nodes.push(u32::from(node.height) | (u32::from(node.missing_links) << 16));
      nodes.push(u32::from(node.high_cover) | (u32::from(node.low_cover) << 16));
    }

    let create = |label: &str, words: &[u32]| {
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(if words.is_empty() { &[0u32] } else { words }),
        usage: wgpu::BufferUsages::STORAGE,
      })
    };

    Some(Self {
      cells: create("ai map cells", &cells),
      nodes: create("ai map nodes", &nodes),
      origin: Vec4::new(map.min_x, map.min_z, map.cell_size, map.floor),
      row_length: map.row_length,
      column_length: map.column_length,
      node_count: map.nodes.len() as u32,
      height: map.height,
    })
  }

  /// What the view reads beside the buffers to show `field`.
  pub(crate) fn get_uniform(&self, field: AiMapField) -> AiMapUniform {
    AiMapUniform {
      origin: self.origin,
      cells: UVec4::new(self.row_length, self.column_length, field as u32, self.node_count),
      height: self.height,
      _pad: [0.0; 3],
    }
  }

  pub(crate) fn get_node_count(&self) -> u32 {
    self.node_count
  }
}
