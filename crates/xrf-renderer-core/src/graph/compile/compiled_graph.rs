use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

use xrf_error::{XrfError, XrfResult};

use crate::graph::compile::compiled_pass::CompiledPass;
use crate::graph::compile::encode_group::EncodeGroup;
use crate::graph::compile::transient_slot::TransientSlot;
use crate::graph::execute::{
  ComputeContext, EncoderContext, GraphBindings, GraphPassScope, GraphResolvedTexture, GraphResources, RasterContext,
};
use crate::graph::frame_graph::FrameGraph;
use crate::graph::pool::{TransientBufferKey, TransientPool, TransientTextureKey};
use crate::graph::record::{GraphBufferRecord, GraphPassWork, GraphTextureRecord};
use crate::graph::report::{GraphPassKind, GraphPassReport, GraphReport, GraphTransientReport};
use crate::graph::resource::{GraphBuffer, GraphTexture, GraphTextureDescriptor};

/// A frame graph ready to execute: its surviving passes in order, the render passes they share, the encode groups they
/// are recorded in, and the slot each transient takes in the pool.
pub struct CompiledGraph<'a> {
  pub(crate) textures: Vec<GraphTextureRecord>,
  pub(crate) buffers: Vec<GraphBufferRecord>,
  pub(crate) passes: Vec<CompiledPass<'a>>,
  pub(crate) culled: Vec<&'static str>,
  pub(crate) render_pass_count: usize,
  pub(crate) groups: Vec<EncodeGroup>,
  pub(crate) texture_slots: Vec<Option<TransientSlot<TransientTextureKey>>>,
  pub(crate) buffer_slots: Vec<Option<TransientSlot<TransientBufferKey>>>,
}

impl<'a> CompiledGraph<'a> {
  /// The surviving passes' names, in the order they run.
  pub fn list_pass_names(&self) -> Vec<&'static str> {
    self.passes.iter().map(|pass| pass.pass.name).collect()
  }

  /// The culled passes' names, in the order they were declared.
  pub fn get_culled(&self) -> &[&'static str] {
    &self.culled
  }

  /// Render passes the raster passes share.
  pub fn get_render_pass_count(&self) -> usize {
    self.render_pass_count
  }

  pub fn get_groups(&self) -> &[EncodeGroup] {
    &self.groups
  }

  /// Where a transient texture lives; `None` for an import or a texture no surviving pass names.
  pub fn get_texture_slot(&self, texture: GraphTexture) -> Option<&TransientSlot<TransientTextureKey>> {
    self.texture_slots[texture.get_index()].as_ref()
  }

  /// Where a transient buffer lives; `None` for an import or a buffer no surviving pass names.
  pub fn get_buffer_slot(&self, buffer: GraphBuffer) -> Option<&TransientSlot<TransientBufferKey>> {
    self.buffer_slots[buffer.get_index()].as_ref()
  }

  /// What the compile made of the graph, for a person or a tool to read.
  pub fn describe(&self) -> GraphReport {
    let group_of = |position: usize| -> &'static str {
      self
        .groups
        .iter()
        .find(|group| group.passes.contains(&position))
        .map_or(FrameGraph::DEFAULT_GROUP, |group| group.name)
    };
    let passes: Vec<GraphPassReport> = self
      .passes
      .iter()
      .enumerate()
      .map(|(position, pass)| GraphPassReport {
        name: pass.pass.name.to_string(),
        kind: GraphPassKind::of(&pass.pass.work),
        group: group_of(position).to_string(),
        render_pass: pass.render_pass,
      })
      .collect();
    let textures = self.texture_slots.iter().enumerate().filter_map(|(index, slot)| {
      slot.map(|slot| GraphTransientReport {
        label: self.textures[index].descriptor.label.to_string(),
        is_texture: true,
        ordinal: slot.ordinal,
        first: slot.first,
        last: slot.last,
        bytes: self.textures[index].descriptor.get_byte_size(),
      })
    });
    let buffers = self.buffer_slots.iter().enumerate().filter_map(|(index, slot)| {
      slot.map(|slot| GraphTransientReport {
        label: self.buffers[index].descriptor.label.to_string(),
        is_texture: false,
        ordinal: slot.ordinal,
        first: slot.first,
        last: slot.last,
        bytes: self.buffers[index].descriptor.size,
      })
    });
    let transients: Vec<GraphTransientReport> = textures.chain(buffers).collect();
    let pooled_textures: HashMap<(TransientTextureKey, usize), u64> = self
      .texture_slots
      .iter()
      .enumerate()
      .filter_map(|(index, slot)| {
        slot.map(|slot| {
          (
            (slot.key, slot.ordinal),
            self.textures[index].descriptor.get_byte_size(),
          )
        })
      })
      .collect();
    let pooled_buffers: HashMap<(TransientBufferKey, usize), u64> = self
      .buffer_slots
      .iter()
      .filter_map(|slot| slot.map(|slot| ((slot.key, slot.ordinal), slot.key.size)))
      .collect();

    GraphReport {
      passes,
      culled: self.culled.iter().map(|name| name.to_string()).collect(),
      groups: self.groups.iter().map(|group| group.name.to_string()).collect(),
      render_pass_count: self.render_pass_count,
      transient_bytes: transients.iter().map(|transient| transient.bytes).sum(),
      pooled_count: pooled_textures.len() + pooled_buffers.len(),
      pooled_bytes: pooled_textures.values().sum::<u64>() + pooled_buffers.values().sum::<u64>(),
      transients,
    }
  }

  /// Records the frame: draws the transients from the pool, resolves the imports from their bindings, and records each
  /// encode group into a command buffer of its own, returned in order for one submit.
  ///
  /// # Errors
  ///
  /// Returns an error for an import a surviving pass names that is not bound, or bound to a resource of another size,
  /// format or too little usage.
  pub fn execute(
    self,
    device: &wgpu::Device,
    pool: &mut TransientPool,
    bindings: &GraphBindings<'_>,
  ) -> XrfResult<Vec<wgpu::CommandBuffer>> {
    pool.begin_frame();

    for (key, count) in Self::count_slots(&self.texture_slots) {
      pool.reserve_textures(device, key, count);
    }

    for (key, count) in Self::count_slots(&self.buffer_slots) {
      pool.reserve_buffers(device, key, count);
    }

    let pool: &TransientPool = pool;
    let resources: GraphResources<'_> = self.resolve(pool, bindings)?;
    let CompiledGraph {
      textures,
      passes,
      groups,
      ..
    } = self;
    let mut pending: VecDeque<CompiledPass<'a>> = passes.into();
    let mut commands: Vec<wgpu::CommandBuffer> = Vec::with_capacity(groups.len());

    for group in &groups {
      let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(group.name),
      });
      let mut remaining: usize = group.passes.len();

      while remaining > 0 {
        let Some(pass) = pending.pop_front() else {
          break;
        };

        remaining -= 1;

        match pass.render_pass {
          Some(render_pass) => {
            let mut run: Vec<CompiledPass<'a>> = vec![pass];

            while remaining > 0
              && pending
                .front()
                .is_some_and(|next| next.render_pass == Some(render_pass))
            {
              run.extend(pending.pop_front());
              remaining -= 1;
            }

            Self::record_render_pass(&mut encoder, run, &textures, &resources);
          }
          None => Self::record_pass(&mut encoder, pass, &resources),
        }
      }

      commands.push(encoder.finish());
    }

    Ok(commands)
  }

  fn count_slots<K: Copy + Eq + std::hash::Hash>(slots: &[Option<TransientSlot<K>>]) -> HashMap<K, usize> {
    let mut counts: HashMap<K, usize> = HashMap::new();

    for slot in slots.iter().flatten() {
      let count: &mut usize = counts.entry(slot.key).or_default();

      *count = (*count).max(slot.ordinal + 1);
    }

    counts
  }

  fn resolve<'r>(&self, pool: &'r TransientPool, bindings: &GraphBindings<'r>) -> XrfResult<GraphResources<'r>> {
    let mut used_textures: Vec<bool> = vec![false; self.textures.len()];
    let mut used_buffers: Vec<bool> = vec![false; self.buffers.len()];

    for pass in &self.passes {
      for texture in &pass.textures {
        used_textures[texture.get_index()] = true;
      }

      for buffer in &pass.buffers {
        used_buffers[buffer.get_index()] = true;
      }
    }

    let mut textures: Vec<Option<GraphResolvedTexture<'r>>> = vec![None; self.textures.len()];
    let mut buffers: Vec<Option<&'r wgpu::Buffer>> = vec![None; self.buffers.len()];

    for (index, record) in self.textures.iter().enumerate() {
      if !used_textures[index] {
        continue;
      }

      textures[index] = Some(if record.is_imported {
        let bound: GraphResolvedTexture<'r> = *bindings.textures.get(&GraphTexture::new(index)).ok_or_else(|| {
          XrfError::new_invalid_error(format!("Imported texture '{}' is not bound", record.descriptor.label))
        })?;

        Self::validate_texture_binding(record, bound.texture)?;
        bound
      } else {
        let slot: &TransientSlot<TransientTextureKey> = self.texture_slots[index]
          .as_ref()
          .expect("every used transient texture has a slot");
        let pooled = pool.get_texture(&slot.key, slot.ordinal);

        GraphResolvedTexture {
          texture: &pooled.texture,
          view: &pooled.view,
        }
      });
    }

    for (index, record) in self.buffers.iter().enumerate() {
      if !used_buffers[index] {
        continue;
      }

      buffers[index] = Some(if record.is_imported {
        let bound: &'r wgpu::Buffer = bindings.buffers.get(&GraphBuffer::new(index)).copied().ok_or_else(|| {
          XrfError::new_invalid_error(format!("Imported buffer '{}' is not bound", record.descriptor.label))
        })?;

        if !bound.usage().contains(record.usage) || bound.size() < record.descriptor.size {
          return Err(XrfError::new_invalid_error(format!(
            "Imported buffer '{}' is bound to one of {} bytes with usage {:?}, where {} bytes with {:?} are declared",
            record.descriptor.label,
            bound.size(),
            bound.usage(),
            record.descriptor.size,
            record.usage
          )));
        }

        bound
      } else {
        let slot: &TransientSlot<TransientBufferKey> = self.buffer_slots[index]
          .as_ref()
          .expect("every used transient buffer has a slot");

        &pool.get_buffer(&slot.key, slot.ordinal).buffer
      });
    }

    Ok(GraphResources { textures, buffers })
  }

  fn validate_texture_binding(record: &GraphTextureRecord, texture: &wgpu::Texture) -> XrfResult {
    let descriptor: &GraphTextureDescriptor = &record.descriptor;

    if texture.size() != descriptor.size
      || texture.format() != descriptor.format
      || texture.mip_level_count() != descriptor.mip_level_count
      || !texture.usage().contains(record.usage)
    {
      return Err(XrfError::new_invalid_error(format!(
        "Imported texture '{}' is bound to a {:?} {:?} texture of {} mips with usage {:?}, where {:?} {:?} of {} mips \
         with {:?} is declared",
        descriptor.label,
        texture.size(),
        texture.format(),
        texture.mip_level_count(),
        texture.usage(),
        descriptor.size,
        descriptor.format,
        descriptor.mip_level_count,
        record.usage
      )));
    }

    Ok(())
  }

  /// Opens one render pass on the run's attachments, as its first pass declares them, and records every pass of the
  /// run into it.
  fn record_render_pass(
    encoder: &mut wgpu::CommandEncoder,
    run: Vec<CompiledPass<'a>>,
    textures: &[GraphTextureRecord],
    resources: &GraphResources<'_>,
  ) {
    let first: &GraphPassWork<'a> = &run[0].pass.work;
    let view_of = |texture: GraphTexture, mip_level: u32, array_layer: u32| -> Cow<'_, wgpu::TextureView> {
      let resolved: GraphResolvedTexture<'_> = resources.get_texture(texture);
      let descriptor: &GraphTextureDescriptor = &textures[texture.get_index()].descriptor;

      if descriptor.mip_level_count == 1 && descriptor.size.depth_or_array_layers == 1 {
        Cow::Borrowed(resolved.view)
      } else {
        Cow::Owned(resolved.texture.create_view(&wgpu::TextureViewDescriptor {
          label: Some(descriptor.label),
          dimension: Some(wgpu::TextureViewDimension::D2),
          base_mip_level: mip_level,
          mip_level_count: Some(1),
          base_array_layer: array_layer,
          array_layer_count: Some(1),
          ..Default::default()
        }))
      }
    };
    let color_views: Vec<Cow<'_, wgpu::TextureView>> = first
      .get_colors()
      .iter()
      .map(|color| view_of(color.texture, color.mip_level, color.array_layer))
      .collect();
    let depth_view: Option<Cow<'_, wgpu::TextureView>> = first
      .get_depth()
      .map(|depth| view_of(depth.texture, depth.mip_level, depth.array_layer));
    let color_attachments: Vec<Option<wgpu::RenderPassColorAttachment<'_>>> = first
      .get_colors()
      .iter()
      .zip(&color_views)
      .map(|(color, view)| {
        Some(wgpu::RenderPassColorAttachment {
          view: view.as_ref(),
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: color.load,
            store: wgpu::StoreOp::Store,
          },
        })
      })
      .collect();
    let depth_stencil_attachment =
      first
        .get_depth()
        .zip(depth_view.as_ref())
        .map(|(depth, view)| wgpu::RenderPassDepthStencilAttachment {
          view: view.as_ref(),
          depth_ops: (!depth.is_read_only).then_some(wgpu::Operations {
            load: depth.load,
            store: wgpu::StoreOp::Store,
          }),
          stencil_ops: None,
        });
    let mut render_pass: wgpu::RenderPass<'static> = encoder
      .begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(run[0].pass.name),
        color_attachments: &color_attachments,
        depth_stencil_attachment,
        ..Default::default()
      })
      .forget_lifetime();

    for pass in run {
      let CompiledPass {
        pass,
        textures,
        buffers,
        ..
      } = pass;

      if let GraphPassWork::Raster { record, .. } = pass.work {
        record(&mut RasterContext {
          pass: &mut render_pass,
          scope: GraphPassScope {
            name: pass.name,
            resources,
            textures: &textures,
            buffers: &buffers,
          },
        });
      }
    }
  }

  fn record_pass(encoder: &mut wgpu::CommandEncoder, pass: CompiledPass<'a>, resources: &GraphResources<'_>) {
    let CompiledPass {
      pass,
      textures,
      buffers,
      ..
    } = pass;
    let scope: GraphPassScope<'_> = GraphPassScope {
      name: pass.name,
      resources,
      textures: &textures,
      buffers: &buffers,
    };

    match pass.work {
      GraphPassWork::Compute { record } => {
        let mut compute_pass: wgpu::ComputePass<'static> = encoder
          .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(pass.name),
            timestamp_writes: None,
          })
          .forget_lifetime();

        record(&mut ComputeContext {
          pass: &mut compute_pass,
          scope,
        });
      }
      GraphPassWork::Encoder { record, .. } => record(&mut EncoderContext { encoder, scope }),
      GraphPassWork::Raster { .. } => unreachable!("a raster pass always has a render pass"),
    }
  }
}
