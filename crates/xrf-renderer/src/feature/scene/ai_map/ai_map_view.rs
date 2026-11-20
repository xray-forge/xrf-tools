use glam::UVec4;
use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, StorageArray};

use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::scene::ai_map::ai_map_field::AiMapField;
use crate::feature::scene::ai_map::ai_map_level::AiMapLevel;
use crate::feature::scene::ai_map::ai_map_parameters::AiMapParameters;
use crate::feature::scene::ai_map::ai_map_pass::AiMapPass;
use crate::feature::scene::ai_map::ai_map_uniform::AiMapUniform;
use crate::feature::view::view_feature::ViewFeature;
use crate::scene::level::view::record::view_record::ViewRecord;

/// A view's showing of the level's AI map over the ground, for its debug views.
#[derive(Default)]
pub(crate) struct AiMapView {
  /// The field the view's debug view shows, none while it shows no node.
  field: Option<AiMapField>,
}

impl AiMapView {
  /// Takes the field `options`' debug view shows this frame.
  pub(crate) fn prepare(&mut self, options: &RenderViewOptions) {
    self.field = AiMapField::of(options.mode.debug_view);
  }

  /// Declares the map's view where the view shows it: each drawn pixel's node field of `level` written over `light`,
  /// nothing where no node stands or the level has no map.
  pub(crate) fn add_node_field<'a>(
    &self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes, view, handles, ..
    }: ViewRecord<'a>,
    level: Option<&'a AiMapLevel>,
    light: GraphTexture,
  ) {
    let Some(field) = self.field else {
      return;
    };
    let uniform: AiMapUniform = level.map_or_else(
      || AiMapUniform {
        cells: UVec4::new(0, 0, field as u32, 0),
        ..AiMapUniform::default()
      },
      |level| level.get_uniform(field),
    );
    let pass: &'a AiMapPass = &passes.scene.ai_map;
    let empty: &'a wgpu::Buffer = pass.get_empty();
    let parameters: AiMapParameters = AiMapParameters {
      depth_target: handles.depth,
      cells: StorageArray::new(bindings.import_buffer(
        &mut *graph,
        "ai map cells",
        level.map_or(empty, |level| &level.cells),
      )),
      nodes: StorageArray::new(bindings.import_buffer(
        &mut *graph,
        "ai map nodes",
        level.map_or(empty, |level| &level.nodes),
      )),
      ai_map: runtime.push_uniform(&uniform),
    };

    graph
      .add_raster_pass("ai map")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(light, wgpu::LoadOp::Load))
      .record(move |context| pass.record(context, view, &parameters));
  }
}

impl ViewFeature for AiMapView {
  fn release(&mut self) {
    self.field = None;
  }
}
