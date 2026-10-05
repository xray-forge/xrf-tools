use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;

use xrf_error::{XrfError, XrfResult};

use crate::graph::access::{GraphColorAttachment, GraphDepthAttachment};
use crate::graph::compile::compiled_graph::CompiledGraph;
use crate::graph::compile::compiled_pass::CompiledPass;
use crate::graph::compile::encode_group::EncodeGroup;
use crate::graph::compile::graph_compile_options::GraphCompileOptions;
use crate::graph::compile::transient_slot::TransientSlot;
use crate::graph::frame_graph::FrameGraph;
use crate::graph::pool::{TransientBufferKey, TransientTextureKey};
use crate::graph::record::{GraphBufferRecord, GraphPass, GraphPassWork, GraphTextureRecord};
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// Turns a declared frame graph into one ready to execute.
pub(crate) struct GraphCompiler;

impl GraphCompiler {
  pub fn compile<'a>(graph: FrameGraph<'a>, options: &GraphCompileOptions) -> XrfResult<CompiledGraph<'a>> {
    let FrameGraph {
      textures,
      buffers,
      passes,
      groups,
      ..
    } = graph;

    for pass in &passes {
      Self::validate_pass(pass, &textures)?;
    }

    let alive: Vec<bool> = if options.is_culling {
      Self::cull(&passes, &textures, &buffers)
    } else {
      vec![true; passes.len()]
    };
    let mut culled: Vec<&'static str> = Vec::new();
    let mut kept: Vec<GraphPass<'a>> = Vec::with_capacity(passes.len());

    for (pass, is_alive) in passes.into_iter().zip(alive) {
      if is_alive {
        kept.push(pass);
      } else {
        culled.push(pass.name);
      }
    }

    Self::validate_order(&kept, &textures, &buffers)?;

    let texture_slots: Vec<Option<TransientSlot<TransientTextureKey>>> = Self::assign_slots(
      &kept,
      textures.len(),
      |pass| pass.list_textures().map(GraphTexture::get_index).collect(),
      |index| {
        let record: &GraphTextureRecord = &textures[index];

        (!record.is_imported).then(|| TransientTextureKey::new(&record.descriptor, record.usage))
      },
      options.is_pooling,
    );
    let buffer_slots: Vec<Option<TransientSlot<TransientBufferKey>>> = Self::assign_slots(
      &kept,
      buffers.len(),
      |pass| pass.list_buffers().map(GraphBuffer::get_index).collect(),
      |index| {
        let record: &GraphBufferRecord = &buffers[index];

        (!record.is_imported).then_some(TransientBufferKey {
          size: record.descriptor.size,
          usage: record.usage,
        })
      },
      options.is_pooling,
    );
    let (render_passes, render_pass_count) = Self::merge(&kept, options.is_merging);
    let encode_groups: Vec<EncodeGroup> = Self::group(&kept, &groups, options.is_grouping);
    let passes: Vec<CompiledPass<'a>> = kept
      .into_iter()
      .zip(render_passes)
      .map(|(pass, render_pass)| CompiledPass {
        textures: pass.list_textures().collect::<BTreeSet<_>>().into_iter().collect(),
        buffers: pass.list_buffers().collect::<BTreeSet<_>>().into_iter().collect(),
        render_pass,
        pass,
      })
      .collect();

    Ok(CompiledGraph {
      textures,
      buffers,
      passes,
      culled,
      render_pass_count,
      groups: encode_groups,
      texture_slots,
      buffer_slots,
    })
  }

  /// A raster pass draws into attachments of one size it neither writes otherwise nor names twice.
  fn validate_pass(pass: &GraphPass<'_>, textures: &[GraphTextureRecord]) -> XrfResult {
    let GraphPassWork::Raster { colors, depth, .. } = &pass.work else {
      return Ok(());
    };

    if colors.is_empty() && depth.is_none() {
      return Err(XrfError::new_invalid_error(format!(
        "Raster pass '{}' has no attachment to draw into",
        pass.name
      )));
    }

    let targets: Vec<(GraphTexture, u32, u32)> = colors
      .iter()
      .map(|color| (color.texture, color.mip_level, color.array_layer))
      .chain(
        depth
          .iter()
          .map(|depth| (depth.texture, depth.mip_level, depth.array_layer)),
      )
      .collect();
    let mut size: Option<(u32, u32)> = None;

    for (texture, mip_level, array_layer) in &targets {
      let record: &GraphTextureRecord = &textures[texture.get_index()];
      let descriptor = &record.descriptor;

      if *mip_level >= descriptor.mip_level_count || *array_layer >= descriptor.size.depth_or_array_layers {
        return Err(XrfError::new_invalid_error(format!(
          "Raster pass '{}' draws into mip {mip_level} layer {array_layer} of '{}', which has {} mips and {} layers",
          pass.name, descriptor.label, descriptor.mip_level_count, descriptor.size.depth_or_array_layers
        )));
      }

      let extent: (u32, u32) = (
        (descriptor.size.width >> mip_level).max(1),
        (descriptor.size.height >> mip_level).max(1),
      );

      match size {
        Some(expected) if expected != extent => {
          return Err(XrfError::new_invalid_error(format!(
            "Raster pass '{}' draws into attachments of different sizes: {}x{} and {}x{} ('{}')",
            pass.name, expected.0, expected.1, extent.0, extent.1, descriptor.label
          )));
        }
        _ => size = Some(extent),
      }

      if pass
        .textures
        .iter()
        .any(|(accessed, access)| accessed == texture && access.is_write())
      {
        return Err(XrfError::new_invalid_error(format!(
          "Raster pass '{}' writes its own attachment '{}' otherwise than by drawing",
          pass.name, descriptor.label
        )));
      }
    }

    for (index, (texture, mip_level, array_layer)) in targets.iter().enumerate() {
      if targets[..index].contains(&(*texture, *mip_level, *array_layer)) {
        return Err(XrfError::new_invalid_error(format!(
          "Raster pass '{}' names attachment '{}' twice",
          pass.name,
          textures[texture.get_index()].descriptor.label
        )));
      }
    }

    Ok(())
  }

  /// Which passes survive: walking back from the last, a pass lives when it is kept, is a bridge, writes an import, or
  /// writes something a living pass after it reads.
  fn cull(passes: &[GraphPass<'_>], textures: &[GraphTextureRecord], buffers: &[GraphBufferRecord]) -> Vec<bool> {
    let mut needed_textures: Vec<bool> = vec![false; textures.len()];
    let mut needed_buffers: Vec<bool> = vec![false; buffers.len()];
    let mut alive: Vec<bool> = vec![false; passes.len()];

    for (index, pass) in passes.iter().enumerate().rev() {
      let is_bridge: bool = matches!(pass.work, GraphPassWork::Encoder { is_bridge: true, .. });
      let is_writing_needed: bool = pass
        .list_texture_writes()
        .any(|texture| textures[texture.get_index()].is_imported || needed_textures[texture.get_index()])
        || pass
          .list_buffer_writes()
          .any(|buffer| buffers[buffer.get_index()].is_imported || needed_buffers[buffer.get_index()]);

      if !(pass.is_kept || is_bridge || is_writing_needed) {
        continue;
      }

      alive[index] = true;

      for texture in pass.list_texture_reads() {
        needed_textures[texture.get_index()] = true;
      }

      for buffer in pass.list_buffer_reads() {
        needed_buffers[buffer.get_index()] = true;
      }
    }

    alive
  }

  /// No surviving pass reads a transient before a pass before it, or it itself, has written it.
  fn validate_order(
    passes: &[GraphPass<'_>],
    textures: &[GraphTextureRecord],
    buffers: &[GraphBufferRecord],
  ) -> XrfResult {
    let mut written_textures: Vec<bool> = textures.iter().map(|record| record.is_imported).collect();
    let mut written_buffers: Vec<bool> = buffers.iter().map(|record| record.is_imported).collect();

    for pass in passes {
      if let Some(texture) = pass
        .list_texture_reads()
        .find(|texture| !written_textures[texture.get_index()])
      {
        return Err(XrfError::new_invalid_error(format!(
          "Pass '{}' reads transient texture '{}' before anything writes it",
          pass.name,
          textures[texture.get_index()].descriptor.label
        )));
      }

      if let Some(buffer) = pass
        .list_buffer_reads()
        .find(|buffer| !written_buffers[buffer.get_index()])
      {
        return Err(XrfError::new_invalid_error(format!(
          "Pass '{}' reads transient buffer '{}' before anything writes it",
          pass.name,
          buffers[buffer.get_index()].descriptor.label
        )));
      }

      for texture in pass.list_texture_writes() {
        written_textures[texture.get_index()] = true;
      }

      for buffer in pass.list_buffer_writes() {
        written_buffers[buffer.get_index()] = true;
      }
    }

    Ok(())
  }

  /// Gives every transient a surviving pass names a slot of its key: pooled, the first whose last user came before
  /// its own first; otherwise one of its own.
  fn assign_slots<K: Copy + Eq + Hash>(
    passes: &[GraphPass<'_>],
    count: usize,
    list: impl Fn(&GraphPass<'_>) -> Vec<usize>,
    key_of: impl Fn(usize) -> Option<K>,
    is_pooling: bool,
  ) -> Vec<Option<TransientSlot<K>>> {
    let mut spans: Vec<Option<(usize, usize)>> = vec![None; count];

    for (position, pass) in passes.iter().enumerate() {
      for index in list(pass) {
        let span: &mut Option<(usize, usize)> = &mut spans[index];

        *span = Some(span.map_or((position, position), |(first, _)| (first, position)));
      }
    }

    let mut order: Vec<usize> = (0..count).filter(|index| spans[*index].is_some()).collect();

    order.sort_by_key(|index| spans[*index].map(|(first, _)| first));

    let mut ends: HashMap<K, Vec<usize>> = HashMap::new();
    let mut slots: Vec<Option<TransientSlot<K>>> = vec![None; count];

    for index in order {
      let (Some(key), Some((first, last))) = (key_of(index), spans[index]) else {
        continue;
      };
      let key_ends: &mut Vec<usize> = ends.entry(key).or_default();
      let ordinal: usize = match key_ends.iter().position(|end| is_pooling && *end < first) {
        Some(ordinal) => {
          key_ends[ordinal] = last;
          ordinal
        }
        None => {
          key_ends.push(last);
          key_ends.len() - 1
        }
      };

      slots[index] = Some(TransientSlot {
        key,
        ordinal,
        first,
        last,
      });
    }

    slots
  }

  /// Gives each raster pass the render pass it draws in: the one before it in the same group when it targets the
  /// same attachments, loads them rather than clearing, and does not sample one of them.
  fn merge(passes: &[GraphPass<'_>], is_merging: bool) -> (Vec<Option<usize>>, usize) {
    let mut render_passes: Vec<Option<usize>> = vec![None; passes.len()];
    let mut count: usize = 0;
    let mut start: Option<usize> = None;

    for (index, pass) in passes.iter().enumerate() {
      if !pass.work.is_raster() {
        start = None;
        continue;
      }

      match start {
        Some(first) if is_merging && Self::is_mergeable(&passes[first], pass) => {
          render_passes[index] = render_passes[first];
        }
        _ => {
          render_passes[index] = Some(count);
          count += 1;
          start = Some(index);
        }
      }
    }

    (render_passes, count)
  }

  fn is_mergeable(first: &GraphPass<'_>, next: &GraphPass<'_>) -> bool {
    let (colors, depth): (&[GraphColorAttachment], Option<&GraphDepthAttachment>) =
      (first.work.get_colors(), first.work.get_depth());
    let (next_colors, next_depth) = (next.work.get_colors(), next.work.get_depth());
    let is_same_colors: bool = colors.len() == next_colors.len()
      && colors
        .iter()
        .zip(next_colors)
        .all(|(color, next)| color.is_same_target(next) && matches!(next.load, wgpu::LoadOp::Load));
    let is_same_depth: bool = match (depth, next_depth) {
      (None, None) => true,
      (Some(depth), Some(next)) => depth.is_same_target(next) && matches!(next.load, wgpu::LoadOp::Load),
      _ => false,
    };
    let is_sampling_target: bool = next.textures.iter().any(|(texture, _)| {
      colors.iter().any(|color| color.texture == *texture) || depth.is_some_and(|depth| depth.texture == *texture)
    });

    first.group == next.group && is_same_colors && is_same_depth && !is_sampling_target
  }

  /// Cuts the surviving passes into encode groups: runs of one declared group, or the whole frame as one.
  fn group(passes: &[GraphPass<'_>], names: &[&'static str], is_grouping: bool) -> Vec<EncodeGroup> {
    if passes.is_empty() {
      return Vec::new();
    }

    if !is_grouping {
      return vec![EncodeGroup {
        name: FrameGraph::DEFAULT_GROUP,
        passes: 0..passes.len(),
      }];
    }

    let mut groups: Vec<EncodeGroup> = Vec::new();

    for (index, pass) in passes.iter().enumerate() {
      match groups.last_mut() {
        Some(group) if passes[group.passes.start].group == pass.group => group.passes.end = index + 1,
        _ => groups.push(EncodeGroup {
          name: names[pass.group],
          passes: index..index + 1,
        }),
      }
    }

    groups
  }
}
