use std::cell::Cell;

use glam::Mat4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::context::gpu_requirements::can_trace;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_indirect_light_settings::RenderIndirectLightSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::history::History;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::bitmask_pass_names::BitmaskPassNames;
use crate::pass::bitmask_search::BitmaskSearch;
use crate::pass::level_passes::LevelPasses;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::vbao_parameters::VbaoParameters;
use crate::pass::vbao_pass::VbaoPass;
use crate::pass::vbao_uniform::VbaoUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::bitmask_tracing::BitmaskTracing;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::view_frame_end::ViewFrameEnd;

/// The visibility-bitmask search at half the frame's size, for VBAO's occlusion, the indirect light or both, RTAO
/// tracing the occlusion in its place where asked: what it searches for this frame, what its passes read, and its
/// accumulations over frames, the occlusion's and, while it gathers light, the light's.
#[derive(Default)]
pub struct BitmaskFeature {
  search: Option<BitmaskSearch>,
  indirect_light: RenderIndirectLightSettings,
  uniform: VbaoUniform,
  /// Visibility, distance along the view and frames gathered, while it accumulates.
  occlusion: Option<History<FrameTexture>>,
  /// The indirect light, while it accumulates and gathers light.
  light: Option<History<FrameTexture>>,
  /// Which of `occlusion` and `light` this frame's passes write, noted as they are added: a frame whose search a
  /// trace answers raw, or the indirect light's own trace replaces, writes neither or only the first.
  written: Cell<[bool; 2]>,
}

impl BitmaskFeature {
  /// Takes this frame's search over `size`, half the frame's: its accumulations made while it accumulates, the light's
  /// while it gathers light too, dropped otherwise, and what its passes read.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, projection: Mat4, size: (u32, u32)) {
    let settings: RenderAmbientOcclusionSettings = options.features.ambient_occlusion;

    self.search = BitmaskSearch::new(options, can_trace(device));
    self.written.set([false; 2]);
    self.indirect_light = options.features.indirect_light;

    let is_lit: bool = self.search.is_some_and(|search| search.is_lit);
    let is_accumulated: bool = self.search.is_some() && settings.vbao.get_accumulation() > 1;
    let make = |label: [&'static str; 2]| {
      move |side: usize| {
        FrameTexture::new(
          device,
          label[side],
          size,
          VbaoPass::HISTORY,
          wgpu::TextureUsages::empty(),
        )
      }
    };
    let is_sized = |history: &History<FrameTexture>| history.get_written().get_size() == size;

    if !is_accumulated || self.occlusion.as_ref().is_some_and(|history| !is_sized(history)) {
      self.occlusion = None;
    }

    if !(is_accumulated && is_lit) || self.light.as_ref().is_some_and(|history| !is_sized(history)) {
      self.light = None;
    }

    if is_accumulated && is_lit && self.light.is_none() {
      self.light = Some(History::new(make([
        "indirect light history 0",
        "indirect light history 1",
      ])));
      // Both start afresh, so the light's frames are counted from its own first.
      self.occlusion = None;
    }

    let history: Option<&History<FrameTexture>> = is_accumulated.then(|| {
      &*History::fit(
        &mut self.occlusion,
        |_| true,
        make(["vbao history 0", "vbao history 1"]),
      )
    });
    let has_light_history: bool = self.light.as_ref().is_some_and(History::is_valid);

    self.uniform = VbaoUniform::new(
      &settings,
      if is_lit {
        (
          self.indirect_light.intensity,
          self.indirect_light.radius,
          self.indirect_light.get_steps(),
        )
      } else {
        (0.0, 0.0, 0)
      },
      projection,
      size,
      history.map_or(0, History::get_frame),
      (history.is_some_and(History::is_valid), has_light_history),
    );
  }

  /// What it searches for this frame, none where it does not run.
  pub fn get_search(&self) -> Option<BitmaskSearch> {
    self.search
  }

  /// The indirect light as asked for, which it gathers while its search is lit.
  pub fn get_indirect_light(&self) -> RenderIndirectLightSettings {
    self.indirect_light
  }

  /// Declares the search: the frame's light copied at half size while lit, searched (into the first occlusion target
  /// while it occludes) or, where RTAO traces the occlusion through the scene's trace (`traced`), traced into it and
  /// searched for the light alone; accumulated with the last frame's into its histories while it accumulates, and
  /// filtered (into the second occlusion target, which combine reads). Answers the indirect light filtered while lit;
  /// nothing where it does not run. RTAO asked for before the scene's trace is built searches as VBAO does meanwhile.
  /// Where the indirect light is traced in its place (`is_light_traced`), it searches for the occlusion alone.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
    tracing: BitmaskTracing<'a>,
  ) -> Option<GraphTexture> {
    let BitmaskTracing {
      traced,
      is_light_traced,
    } = tracing;
    let asked: BitmaskSearch = self.search?;
    let trace: Option<(TraceParameters<'a>, &'a wgpu::BindGroup)> = traced.filter(|_| asked.is_traced);
    let search: BitmaskSearch = BitmaskSearch {
      is_traced: trace.is_some(),
      is_lit: asked.is_lit && !is_light_traced,
      ..asked
    };

    if !(search.is_occluding || search.is_lit) {
      return None;
    }

    let names: BitmaskPassNames = search.get_names();
    let is_lit: bool = search.is_lit;
    let occlusion: UniformBinding<VbaoUniform> = runtime.push_uniform(&self.uniform);
    let empty: GraphTexture = bindings.import_view(graph, "vbao history empty", passes.vbao.get_empty());
    let dark: GraphTexture = bindings.import_view(graph, "indirect light none", passes.vbao.get_dark());
    let (width, height) = (targets.size.0.div_ceil(2), targets.size.1.div_ceil(2));
    let mut create = |label: &'static str, format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let points: GraphTexture = create("vbao points", VbaoPass::POINTS);
    // What every stage reads, with nothing before it: each stage names the occlusion before it and its history, the
    // light copied, and the light before it and its history.
    let base: VbaoParameters = VbaoParameters {
      normal_target: targets.normal,
      depth_target: targets.depth,
      motion_target: targets.motion,
      occlusion,
      source: empty,
      history: empty,
      albedo_target: targets.albedo,
      material_target: targets.material,
      light_target: targets.light,
      light_source: dark,
      gathered: dark,
      light_history: dark,
      points,
    };
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));
    // The occlusion targets while it occludes; a search of the light alone has its own, its visibility unused.
    let [searched, filtered] = if search.is_occluding {
      [targets.occlusion[1], targets.occlusion[0]]
    } else {
      [create("indirect search", ViewTargets::OCCLUSION), dark]
    };
    let light_source: GraphTexture = if is_lit {
      create("indirect source", VbaoPass::LIGHT)
    } else {
      dark
    };
    let lit_searched: Option<GraphTexture> = is_lit.then(|| create("indirect gathered", VbaoPass::LIGHT));
    let lit_filtered: Option<GraphTexture> = is_lit.then(|| create("indirect light", VbaoPass::LIGHT));
    // Traced, the search gathers the light alone, its visibility into a target of its own, unused.
    let search_visibility: GraphTexture = if trace.is_some() && is_lit {
      create("indirect search", ViewTargets::OCCLUSION)
    } else {
      searched
    };
    let laying: VbaoParameters = VbaoParameters { points: empty, ..base };

    graph
      .add_raster_pass(VbaoPass::POINTS_PASS)
      .parameters(&laying)
      .color(clear(points))
      .record(move |context| passes.vbao.record_points(context, view, &laying));

    if is_lit {
      let copy: VbaoParameters = base;

      graph
        .add_raster_pass(BitmaskSearch::SOURCE_PASS)
        .parameters(&copy)
        .color(clear(light_source))
        .record(move |context| passes.vbao.record_light_source(context, view, &copy));
    }

    let searching: VbaoParameters = VbaoParameters { light_source, ..base };

    if trace.is_none() || is_lit {
      lit_searched
        .into_iter()
        .fold(
          graph
            .add_raster_pass(names.search)
            .parameters(&searching)
            .color(clear(search_visibility)),
          |builder, light| builder.color(clear(light)),
        )
        .record(move |context| passes.vbao.record_search(context, is_lit, view, &searching));
    }

    if let Some(trace) = trace {
      Self::add_trace_pass(graph, passes, view, (base, trace), searched);
    }

    self.add_gather_passes(
      (graph, bindings),
      passes,
      view,
      (base, search, names),
      (searched, filtered, lit_searched, lit_filtered),
    );

    lit_filtered
  }

  /// Traces the occlusion through the scene's trace (`trace`, its textures bound beside) into `searched`.
  fn add_trace_pass<'a>(
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (base, (trace, textures)): (VbaoParameters, (TraceParameters<'a>, &'a wgpu::BindGroup)),
    searched: GraphTexture,
  ) {
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));
    let tracing: VbaoParameters = base;

    graph
      .add_raster_pass(BitmaskSearch::TRACE_PASS)
      .parameters(&tracing)
      .parameters(&trace)
      .color(clear(searched))
      .record(move |context| passes.rtao.record(context, view, (&tracing, &trace, textures)));
  }

  /// Accumulates the search with the last frame's into its histories while it accumulates, then filters it into the
  /// occlusion target combine reads (`filtered`) and the indirect light (`lit_filtered`) while lit; `base` the stages'
  /// parameters with nothing read, `searched` and `lit_searched` what the search wrote.
  fn add_gather_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (base, search, names): (VbaoParameters, BitmaskSearch, BitmaskPassNames),
    (searched, filtered_occlusion, lit_searched, lit_filtered): (
      GraphTexture,
      GraphTexture,
      Option<GraphTexture>,
      Option<GraphTexture>,
    ),
  ) {
    let (dark, is_lit): (GraphTexture, bool) = (base.light_source, search.is_lit);
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));
    // The last frame's accumulations and this frame's, while it accumulates.
    let mut import = |label: &'static str, history: &'a History<FrameTexture>| {
      [
        bindings.import_view(&mut *graph, label, &history.get_read().view),
        bindings.import_view(&mut *graph, label, &history.get_written().view),
      ]
    };
    let histories: Option<[GraphTexture; 2]> = self.occlusion.as_ref().map(|history| import("vbao history", history));
    let light_histories: Option<[GraphTexture; 2]> = self
      .light
      .as_ref()
      .filter(|_| self.occlusion.is_some() && is_lit)
      .map(|history| import("indirect light history", history));
    self.written.set([histories.is_some(), light_histories.is_some()]);

    let (gathered, gathered_light): (GraphTexture, Option<GraphTexture>) = match histories {
      Some([previous, written]) => {
        let [previous_light, written_light] = light_histories.unwrap_or([dark, dark]);
        let written_light: Option<GraphTexture> = light_histories.map(|_| written_light);
        let accumulate: VbaoParameters = VbaoParameters {
          source: searched,
          history: previous,
          gathered: lit_searched.unwrap_or(dark),
          light_history: previous_light,
          ..base
        };
        let is_lit_accumulated: bool = written_light.is_some();

        written_light
          .into_iter()
          .fold(
            graph
              .add_raster_pass(names.accumulate)
              .parameters(&accumulate)
              .color(clear(written)),
            |builder, light| builder.color(clear(light)),
          )
          .record(move |context| {
            passes
              .vbao
              .record_accumulate(context, is_lit_accumulated, view, &accumulate)
          });

        (written, written_light.or(lit_searched))
      }
      None => (searched, lit_searched),
    };
    let filtering: VbaoParameters = VbaoParameters {
      source: gathered,
      gathered: gathered_light.unwrap_or(dark),
      ..base
    };

    search
      .is_occluding
      .then_some(filtered_occlusion)
      .into_iter()
      .chain(lit_filtered)
      .fold(
        graph.add_raster_pass(names.filter).parameters(&filtering),
        |builder, target| builder.color(clear(target)),
      )
      .record(move |context| passes.vbao.record_filter(context, search, view, &filtering));
  }
}

impl ViewFeature for BitmaskFeature {
  /// Its accumulations.
  fn get_memory(&self) -> u64 {
    [&self.occlusion, &self.light]
      .into_iter()
      .flatten()
      .flat_map(History::get_sides)
      .map(FrameTexture::get_memory)
      .sum()
  }

  /// The accumulations this frame wrote become the ones the next reads; one it did not write holds nothing to read.
  fn end_frame(&mut self, _end: &mut ViewFrameEnd<'_>) {
    for (history, is_written) in [&mut self.occlusion, &mut self.light]
      .into_iter()
      .zip(self.written.get())
    {
      match history {
        Some(history) if is_written => history.swap(),
        Some(history) => history.invalidate(),
        None => {}
      }
    }
  }

  fn release(&mut self) {
    self.occlusion = None;
    self.light = None;
  }
}
