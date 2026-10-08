use glam::Mat4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
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
use crate::pass::vbao_parameters::VbaoParameters;
use crate::pass::vbao_pass::VbaoPass;
use crate::pass::vbao_uniform::VbaoUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The visibility-bitmask search at half the frame's size, for VBAO's occlusion, the indirect light or both: what it
/// searches for this frame, what its passes read, and its accumulations over frames, the occlusion's and, while it
/// gathers light, the light's.
#[derive(Default)]
pub struct BitmaskFeature {
  search: Option<BitmaskSearch>,
  quality: RenderAmbientOcclusionQuality,
  indirect_light: RenderIndirectLightSettings,
  uniform: VbaoUniform,
  /// Visibility, distance along the view and frames gathered, while it accumulates.
  occlusion: Option<History<FrameTexture>>,
  /// The indirect light, while it accumulates and gathers light.
  light: Option<History<FrameTexture>>,
}

impl BitmaskFeature {
  /// Takes this frame's search over `size`, half the frame's: its accumulations made while it accumulates, the light's
  /// while it gathers light too, dropped otherwise, and what its passes read.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, projection: Mat4, size: (u32, u32)) {
    let settings: RenderAmbientOcclusionSettings = options.features.ambient_occlusion;

    self.search = BitmaskSearch::new(options);
    self.quality = settings.quality;
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
        (self.indirect_light.intensity, self.indirect_light.radius)
      } else {
        (0.0, 0.0)
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
  /// while it occludes), accumulated with the last frame's into its histories while it accumulates, and filtered (into
  /// the second occlusion target, which combine reads). Answers the indirect light filtered while lit; nothing where
  /// it does not run.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
  ) -> Option<GraphTexture> {
    let search: BitmaskSearch = self.search?;
    let quality: RenderAmbientOcclusionQuality = self.quality;
    let names: BitmaskPassNames = search.get_names();
    let is_lit: bool = search.is_lit;
    let occlusion: UniformBinding<VbaoUniform> = runtime.push_uniform(&self.uniform);
    let empty: GraphTexture = bindings.import_view(graph, "vbao history empty", passes.vbao.get_empty());
    let dark: GraphTexture = bindings.import_view(graph, "indirect light none", passes.vbao.get_dark());
    let (width, height) = (targets.size.0.div_ceil(2), targets.size.1.div_ceil(2));
    let mut create = |label: &'static str, format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    // What a stage reads: the occlusion before it and its history, the light copied, and the light before it and its
    // history.
    let points: GraphTexture = create("vbao points", VbaoPass::POINTS);
    let parameters =
      |(source, history): (GraphTexture, GraphTexture),
       (light_source, gathered, light_history): (GraphTexture, GraphTexture, GraphTexture)| {
        VbaoParameters {
          normal_target: targets.normal,
          depth_target: targets.depth,
          motion_target: targets.motion,
          occlusion,
          source,
          history,
          albedo_target: targets.albedo,
          material_target: targets.material,
          light_target: targets.light,
          light_source,
          gathered,
          light_history,
          points,
        }
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

    let laying: VbaoParameters = VbaoParameters {
      points: empty,
      ..parameters((empty, empty), (dark, dark, dark))
    };

    graph
      .add_raster_pass(VbaoPass::POINTS_PASS)
      .parameters(&laying)
      .color(clear(points))
      .record(move |context| passes.vbao.record_points(context, view, &laying));

    if is_lit {
      let copy: VbaoParameters = parameters((empty, empty), (dark, dark, dark));

      graph
        .add_raster_pass(BitmaskSearch::SOURCE_PASS)
        .parameters(&copy)
        .color(clear(light_source))
        .record(move |context| passes.vbao.record_light_source(context, view, &copy));
    }

    let searching: VbaoParameters = parameters((empty, empty), (light_source, dark, dark));

    lit_searched
      .into_iter()
      .fold(
        graph
          .add_raster_pass(names.search)
          .parameters(&searching)
          .color(clear(searched)),
        |builder, light| builder.color(clear(light)),
      )
      .record(move |context| passes.vbao.record_search(context, (quality, is_lit), view, &searching));

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
    let (gathered, gathered_light): (GraphTexture, Option<GraphTexture>) = match histories {
      Some([previous, written]) => {
        let [previous_light, written_light] = light_histories.unwrap_or([dark, dark]);
        let written_light: Option<GraphTexture> = light_histories.map(|_| written_light);
        let accumulate: VbaoParameters = parameters(
          (searched, previous),
          (dark, lit_searched.unwrap_or(dark), previous_light),
        );
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
    let filtering: VbaoParameters = parameters((gathered, empty), (dark, gathered_light.unwrap_or(dark), dark));

    search
      .is_occluding
      .then_some(filtered)
      .into_iter()
      .chain(lit_filtered)
      .fold(
        graph.add_raster_pass(names.filter).parameters(&filtering),
        |builder, target| builder.color(clear(target)),
      )
      .record(move |context| passes.vbao.record_filter(context, search, view, &filtering));

    lit_filtered
  }

  /// The accumulations this frame wrote become the ones the next reads.
  pub fn end_frame(&mut self) {
    for history in [&mut self.occlusion, &mut self.light].into_iter().flatten() {
      history.swap();
    }
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

  fn release(&mut self) {
    self.occlusion = None;
    self.light = None;
  }
}
