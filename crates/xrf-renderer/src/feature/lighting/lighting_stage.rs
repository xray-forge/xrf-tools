use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::contract::render_ambient_occlusion_method::RenderAmbientOcclusionMethod;
use crate::contract::render_applied_indirect_light::RenderAppliedIndirectLight;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::feature::image::resolve::resolve_decisions::ResolveDecisions;
use crate::feature::lighting::ambient_occlusion::ambient_occlusion_feature::AmbientOcclusionFeature;
use crate::feature::lighting::bitmask::bitmask_feature::BitmaskFeature;
use crate::feature::lighting::bitmask::bitmask_tracing::BitmaskTracing;
use crate::feature::lighting::combine::combine_feature::CombineFeature;
use crate::feature::lighting::combine::combine_inputs::CombineInputs;
use crate::feature::lighting::contact_shadows::contact_shadow_feature::ContactShadowFeature;
use crate::feature::lighting::contact_shadows::contact_shadows_recorded::ContactShadowsRecorded;
use crate::feature::lighting::direct_light_recorded::DirectLightRecorded;
use crate::feature::lighting::sun::direct_light_inputs::DirectLightInputs;
use crate::feature::lighting::sun::sun_feature::SunFeature;
use crate::feature::tracing::indirect_trace::indirect_trace_feature::IndirectTraceFeature;
use crate::feature::tracing::indirect_trace::indirect_traced::IndirectTraced;
use crate::feature::tracing::sun_trace::sun_trace_feature::SunTraceFeature;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::scene::level::feature::view_units::view_units;
use crate::scene::level::view_frame::ViewFrame;
use crate::scene::level::view_prepare::ViewPrepare;
use crate::scene::level::view_record::ViewRecord;

/// The units of a view's lighting that draw in it alone: the sun with its contact shadows and its trace, the screen's
/// occlusion and the bitmask search, the traced indirect light and the combine. The lights, the sky, the reflections,
/// the resolve's guides and the water record between them.
#[derive(Default)]
pub struct LightingStage {
  sun: SunFeature,
  sun_trace: SunTraceFeature,
  contact_shadows: ContactShadowFeature,
  ambient_occlusion: AmbientOcclusionFeature,
  bitmask: BitmaskFeature,
  indirect_trace: IndirectTraceFeature,
  combine: CombineFeature,
}

impl LightingStage {
  /// Takes this frame's units from what the view decided (`shared`): the occlusion and the bitmask search at half the
  /// frame's size, the indirect light traced where ray reconstruction denoises it and the search gathers light, the
  /// combine, and the sun's contact shadows and trace by the frame's lighting.
  pub fn prepare(&mut self, frame: &ViewPrepare<'_>, shared: &ViewFrame) {
    let (options, view) = (frame.options, frame.view);
    let (width, height) = frame.drawn;
    let half: (u32, u32) = (width.div_ceil(2), height.div_ceil(2));
    let resolve: ResolveDecisions = shared.resolve;

    self.ambient_occlusion.prepare(options, view.projection, half);
    self.bitmask.prepare(frame, half, resolve.reference_sample);

    let is_light_traced: bool =
      frame.passes.tracing.indirect_trace.is_traced() && resolve.is_ray_reconstructed && self.is_indirect();

    self.indirect_trace.prepare(
      is_light_traced.then(|| self.bitmask.get_indirect_light().intensity),
      resolve.noise_frame,
    );
    self.combine.prepare(&shared.lighting, view.get_depth_range().1);
    self
      .contact_shadows
      .prepare(options, &shared.lighting, height, resolve.noise_frame);
    self.sun_trace.prepare(options, &shared.lighting, resolve);
  }

  /// Whether the screen's occlusion is drawn this frame.
  pub fn is_occluded(&self) -> bool {
    self.ambient_occlusion.is_drawn()
  }

  /// Whether the bitmask search gathers indirect light this frame.
  pub fn is_indirect(&self) -> bool {
    self.bitmask.get_search().is_some_and(|search| search.is_lit)
  }

  /// Writes into `applied` what it draws this frame: the occlusion and how it is found, and the indirect light.
  pub fn describe(&self, applied: &mut RenderAppliedReport) {
    let is_occluded: bool = self.is_occluded();

    applied.ambient_occlusion = is_occluded.then_some(*self.ambient_occlusion.get_settings());
    applied.ambient_occlusion_method = is_occluded.then(|| match self.bitmask.get_search() {
      Some(search) if search.is_occluding && search.is_traced => RenderAmbientOcclusionMethod::Rtao,
      Some(search) if search.is_occluding => RenderAmbientOcclusionMethod::Vbao,
      _ => RenderAmbientOcclusionMethod::Gtao,
    });
    applied.indirect_light =
      self
        .bitmask
        .get_search()
        .filter(|search| search.is_lit)
        .map(|search| RenderAppliedIndirectLight {
          intensity: self.bitmask.get_indirect_light().intensity,
          is_shared: search.is_occluding,
        });
  }

  /// Adds the sun's light: its contact shadows, its trace where DLSS ray reconstruction denoises it, then the sun drawn
  /// into the light target, shadowed by its cascades where it is not traced. Answers how much of the sun reaches each
  /// pixel and what the lights march their contact shadows by.
  pub fn add_direct<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    DirectLightInputs {
      traced,
      wet,
      filter_reach,
    }: DirectLightInputs<'a>,
  ) -> DirectLightRecorded {
    let ViewRecord {
      passes,
      view,
      handles: targets,
      ..
    } = record;
    let contact: ContactShadowsRecorded =
      self
        .contact_shadows
        .add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, view, targets);
    let sun_traced: Option<GraphTexture> = self.sun_trace.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      view,
      targets,
      traced,
    );

    DirectLightRecorded {
      sun_seen: self.sun.add_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        wet,
        contact.sun,
        sun_traced,
        filter_reach,
      ),
      contact: contact.uniform,
      contact_lights: self.contact_shadows.get_light_count(),
    }
  }

  /// Adds the occlusion and the indirect light searched over the screen: GTAO on its own, or VBAO, the indirect light
  /// or both from one bitmask search, RTAO through the scene's trace (`traced`). Answers the indirect light filtered,
  /// none where it is not gathered.
  pub fn add_occlusion<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      handles: targets,
      ..
    }: ViewRecord<'a>,
    traced: Option<TracedScene<'a>>,
  ) -> Option<GraphTexture> {
    self
      .ambient_occlusion
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, view, targets);
    self.bitmask.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      view,
      targets,
      BitmaskTracing {
        traced,
        is_light_traced: self.indirect_trace.is_traced(traced.as_ref()),
      },
    )
  }

  /// Adds the indirect light traced in the search's place where it is this frame, against the `sky` a missed ray reads.
  pub fn add_indirect_trace<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    sky: SkyParameters<'a>,
    traced: Option<TracedScene<'a>>,
  ) -> Option<IndirectTraced> {
    self
      .indirect_trace
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, sky, traced)
  }

  /// Adds the combine: the frame's light, the indirect light and the reflections over the G-buffer under the `sky`.
  pub fn add_combine<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    sky: SkyParameters<'a>,
    inputs: CombineInputs,
  ) {
    self
      .combine
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, sky, inputs);
  }
}

view_units!(LightingStage {
  sun,
  sun_trace,
  contact_shadows,
  ambient_occlusion,
  bitmask,
  indirect_trace,
  combine,
});
