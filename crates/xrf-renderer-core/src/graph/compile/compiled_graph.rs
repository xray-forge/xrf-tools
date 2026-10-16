use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};

use crate::graph::compile::compiled_pass::CompiledPass;
use crate::graph::compile::encode_group::EncodeGroup;
use crate::graph::compile::transient_slot::TransientSlot;
use crate::graph::execute::{
  ComputeContext, EncoderContext, ExecutedGraph, ExecutedGroup, GraphBindings, GraphPassScope, GraphResolvedTexture,
  GraphResources, GraphRuntime, PassMarker, RasterContext,
};
use crate::graph::frame_graph::FrameGraph;
use crate::graph::graph_owner::GraphOwner;
use crate::graph::pool::{AttachmentViewCache, TransientBufferKey, TransientPool, TransientTextureKey};
use crate::graph::record::{GraphBufferRecord, GraphPassWork, GraphTextureRecord};
use crate::graph::report::{GraphPassKind, GraphPassReport, GraphReport, GraphTransientReport};
use crate::graph::resource::{GraphBuffer, GraphTexture, GraphTextureDescriptor};
use crate::graph::timing::{GroupStamps, TimedPass};
use crate::param::BindGroupCache;

/// One encode group as recorded: its command buffers in submit order, what its encoding took, and what wgpu refused of
/// it.
type RecordedGroup = (Vec<wgpu::CommandBuffer>, ExecutedGroup, Option<wgpu::Error>);

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
        owner: pass.pass.owner,
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

  /// Records the frame: uploads what was pushed to the runtime's ring, draws the transients from its pool, resolves the
  /// imports from their bindings, and records each encode group into command buffers of its own, returned in order for
  /// one submit with those its passes handed over, with what each group cost the CPU. While the runtime's timer is
  /// timing, it stamps the start of each group and the end of each pass or render pass; call its `request` after the
  /// submit.
  ///
  /// # Errors
  ///
  /// Returns an error for an import a surviving pass names that is not bound, or bound to a resource of another size,
  /// format or too little usage.
  pub fn execute(
    self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    runtime: &mut GraphRuntime,
    bindings: &GraphBindings<'_>,
  ) -> XrfResult<ExecutedGraph> {
    let GraphRuntime {
      pool,
      bind_groups,
      attachment_views,
      uploads,
      timer,
    } = runtime;

    // The frame's uniforms go up first, so the buffer they are bound from is the one they lie in.
    uploads.flush(device, queue);
    pool.begin_frame();
    bind_groups.begin_frame();
    attachment_views.begin_frame();

    for (key, count) in Self::count_slots(&self.texture_slots) {
      pool.reserve_textures(device, key, count);
    }

    for (key, count) in Self::count_slots(&self.buffer_slots) {
      pool.reserve_buffers(device, key, count);
    }

    let pool: &TransientPool = pool;
    let cache: &BindGroupCache = bind_groups;
    let resources: GraphResources<'_> = self.resolve(pool, bindings, (uploads.get_buffer(), attachment_views))?;
    let CompiledGraph { passes, groups, .. } = self;
    let counts: Vec<u32> = groups
      .iter()
      .map(|group| Self::count_stamps(&passes[group.passes.clone()]))
      .collect();
    let mut stamps: Vec<Option<GroupStamps>> = match timer.begin_frame(device, &counts) {
      Some(stamps) => stamps.into_iter().map(Some).collect(),
      None => groups.iter().map(|_| None).collect(),
    };
    let mut pending: VecDeque<CompiledPass<'a>> = passes.into();
    let batches: Vec<Vec<CompiledPass<'a>>> = groups
      .iter()
      .map(|group| pending.drain(..group.passes.len()).collect())
      .collect();
    let started: Instant = Instant::now();
    let shared = (device, cache, &resources);
    let recorded: Vec<RecordedGroup> = batches
      .into_par_iter()
      .zip(groups.par_iter())
      .zip(stamps.par_iter_mut())
      .map(|((passes, group), stamps)| Self::record_group(group, passes, shared, stamps.as_mut()))
      .collect();
    let errors: Vec<String> = recorded
      .iter()
      .filter_map(|(_, group, error)| error.as_ref().map(|error| format!("group '{}': {error}", group.name)))
      .collect();

    // A frame any group of which wgpu refused is dropped whole: what the others drew is of a frame never finished, and
    // the stamps it recorded are never submitted, so the timer takes none of them.
    if !errors.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "The frame was dropped: {}",
        errors.join("; ")
      )));
    }

    let (commands, executed_groups): (Vec<Vec<wgpu::CommandBuffer>>, Vec<ExecutedGroup>) = recorded
      .into_iter()
      .map(|(commands, group, _)| (commands, group))
      .unzip();
    let commands: Vec<wgpu::CommandBuffer> = commands.into_iter().flatten().collect();
    let encode: Duration = started.elapsed();

    if let Some(stamps) = stamps.into_iter().collect::<Option<Vec<GroupStamps>>>() {
      timer.finish_frame(stamps);
    }

    Ok(ExecutedGraph {
      commands,
      groups: executed_groups,
      encode,
    })
  }

  /// Stamps a group writes when timed: one starting it, and one ending each pass or each run of passes sharing a
  /// render pass.
  fn count_stamps(passes: &[CompiledPass<'_>]) -> u32 {
    let ends: usize = passes
      .iter()
      .enumerate()
      .filter(|(index, pass)| {
        pass.render_pass.is_none() || *index == 0 || passes[*index - 1].render_pass != pass.render_pass
      })
      .count();

    1 + ends as u32
  }

  /// Records one encode group's passes into an encoder of its own, a run of passes sharing a render pass into one, and
  /// finishes it; where a pass hands over commands recorded apart, the encoder is finished there, they follow it, and a
  /// new encoder takes the passes after. `timing` stamps each pass into the group's own range, and resolves the frame's
  /// stamps where the group is the last. What wgpu refused of it, recording or finishing, comes back beside it, caught
  /// on the thread recording it.
  fn record_group(
    group: &EncodeGroup,
    passes: Vec<CompiledPass<'a>>,
    (device, cache, resources): (&wgpu::Device, &BindGroupCache, &GraphResources<'_>),
    mut timing: Option<&mut GroupStamps>,
  ) -> RecordedGroup {
    profiling::scope!("encode group", group.name);

    let recording: Instant = Instant::now();
    let out_of_memory: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let validation: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let begin = || {
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(group.name),
      })
    };
    let mut encoder: wgpu::CommandEncoder = begin();
    let mut commands: Vec<wgpu::CommandBuffer> = Vec::new();
    let mut finished: Duration = Duration::ZERO;
    let mut pending: VecDeque<CompiledPass<'a>> = passes.into();

    if let Some(stamps) = timing.as_mut() {
      stamps.stamp(&mut encoder, None);
    }

    while let Some(pass) = pending.pop_front() {
      let owner: GraphOwner = pass.pass.owner;
      let timed: TimedPass = match pass.render_pass {
        Some(render_pass) => {
          let mut run: Vec<CompiledPass<'a>> = vec![pass];

          while pending
            .front()
            .is_some_and(|next| next.render_pass == Some(render_pass))
          {
            run.extend(pending.pop_front());
          }

          let timed: TimedPass = TimedPass {
            owner,
            passes: run.iter().map(|pass| pass.pass.name).collect(),
          };

          profiling::scope!("record", &timed.to_name());

          Self::record_render_pass(&mut encoder, run, (device, cache, resources));

          timed
        }
        None => {
          let timed: TimedPass = TimedPass {
            owner,
            passes: vec![pass.pass.name],
          };

          profiling::scope!("record", pass.pass.name);

          let (is_marked, handed): (bool, Vec<wgpu::CommandBuffer>) =
            Self::record_pass(&mut encoder, pass, (device, cache, resources), timing.as_deref_mut());

          if !handed.is_empty() {
            let finishing: Instant = Instant::now();

            commands.push(std::mem::replace(&mut encoder, begin()).finish());
            finished += finishing.elapsed();
            commands.extend(handed);
          }

          // A pass that marked its own stages is timed by them.
          if is_marked {
            continue;
          }

          timed
        }
      };

      if let Some(stamps) = timing.as_mut() {
        stamps.stamp(&mut encoder, Some(timed));
      }
    }

    if let Some(stamps) = &timing {
      stamps.finish(&mut encoder);
    }

    let finishing: Instant = Instant::now();

    {
      profiling::scope!("finish", group.name);

      commands.push(encoder.finish());
    }

    // The finishes of encoders a hand-over ended fall inside the recording's span, and count as encoding.
    let recorded: Duration = finishing.duration_since(recording).saturating_sub(finished);

    finished += finishing.elapsed();

    let error: Option<wgpu::Error> = pollster::block_on(validation.pop()).or(pollster::block_on(out_of_memory.pop()));

    (
      commands,
      ExecutedGroup {
        name: group.name,
        record: recorded,
        finish: finished,
      },
      error,
    )
  }

  fn count_slots<K: Copy + Eq + std::hash::Hash>(slots: &[Option<TransientSlot<K>>]) -> HashMap<K, usize> {
    let mut counts: HashMap<K, usize> = HashMap::new();

    for slot in slots.iter().flatten() {
      let count: &mut usize = counts.entry(slot.key).or_default();

      *count = (*count).max(slot.ordinal + 1);
    }

    counts
  }

  fn resolve<'r>(
    &self,
    pool: &'r TransientPool,
    bindings: &GraphBindings<'r>,
    (upload, views): (&'r wgpu::Buffer, &mut AttachmentViewCache),
  ) -> XrfResult<GraphResources<'r>> {
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

    let mut attachments: HashMap<(GraphTexture, u32, u32), wgpu::TextureView> = HashMap::new();

    for pass in &self.passes {
      let work: &GraphPassWork<'_> = &pass.pass.work;
      let targets = work
        .get_colors()
        .iter()
        .map(|color| (color.texture, color.mip_level, color.array_layer))
        .chain(
          work
            .get_depth()
            .map(|depth| (depth.texture, depth.mip_level, depth.array_layer)),
        );

      for (texture, mip_level, array_layer) in targets {
        let descriptor: &GraphTextureDescriptor = &self.textures[texture.get_index()].descriptor;
        let is_whole: bool =
          descriptor.mip_level_count == 1 && descriptor.view_dimension == wgpu::TextureViewDimension::D2;

        if let Some(resolved) = textures[texture.get_index()].filter(|_| !is_whole) {
          attachments
            .entry((texture, mip_level, array_layer))
            .or_insert_with(|| views.get_view(resolved.texture, (mip_level, array_layer), descriptor.label));
        }
      }
    }

    Ok(GraphResources {
      textures,
      attachments,
      buffers,
      upload,
    })
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
    (device, cache, resources): (&wgpu::Device, &BindGroupCache, &GraphResources<'_>),
  ) {
    let first: &GraphPassWork<'a> = &run[0].pass.work;
    let color_attachments: Vec<Option<wgpu::RenderPassColorAttachment<'_>>> = first
      .get_colors()
      .iter()
      .map(|color| {
        Some(wgpu::RenderPassColorAttachment {
          view: resources.get_attachment(color.texture, color.mip_level, color.array_layer),
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: color.load,
            store: wgpu::StoreOp::Store,
          },
        })
      })
      .collect();
    let depth_stencil_attachment = first.get_depth().map(|depth| wgpu::RenderPassDepthStencilAttachment {
      view: resources.get_attachment(depth.texture, depth.mip_level, depth.array_layer),
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
            device,
            cache,
            resources,
            textures: &textures,
            buffers: &buffers,
          },
        });
      }
    }
  }

  /// Records a compute or encoder pass, answering whether it marked stages of its own and with the commands it handed
  /// over.
  fn record_pass(
    encoder: &mut wgpu::CommandEncoder,
    pass: CompiledPass<'a>,
    (device, cache, resources): (&wgpu::Device, &BindGroupCache, &GraphResources<'_>),
    timer: Option<&mut GroupStamps>,
  ) -> (bool, Vec<wgpu::CommandBuffer>) {
    let CompiledPass {
      pass,
      textures,
      buffers,
      ..
    } = pass;
    let scope: GraphPassScope<'_> = GraphPassScope {
      name: pass.name,
      device,
      cache,
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

        (false, Vec::new())
      }
      GraphPassWork::Encoder { record, .. } => {
        let mut context: EncoderContext<'_> = EncoderContext {
          encoder,
          scope,
          marker: PassMarker {
            timer,
            owner: pass.owner,
            is_marked: false,
          },
          handed: Vec::new(),
        };

        record(&mut context);
        (context.marker.is_marked, context.handed)
      }
      GraphPassWork::Raster { .. } => unreachable!("a raster pass always has a render pass"),
    }
  }
}
