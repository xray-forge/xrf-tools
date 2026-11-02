use xrf_renderer_core::{
  GraphBuffer, GraphBufferAccess, GraphDeclaration, GraphDepthAttachment, GraphTexture, StorageArray, StorageArrayMut,
};

use crate::feature::lighting::shadows::shadow_casters::ShadowCasters;
use crate::feature::lighting::shadows::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
use crate::feature::scene::drawable::casters::drawable_kind_ref::DrawableKindRef;
use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::feature::scene::statics::cull::shadow_cull::ShadowCull;
use crate::feature::scene::statics::draw::static_draws::StaticDraws;
use crate::feature::scene::statics::draw::static_layout_bindings::StaticLayoutBindings;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::build::indirect_args::IndirectArgs;
use crate::scene::level::lights::atlas::level_light_shadows::{
  FACE_PASSES, FaceDraw, FaceDue, FaceSlot, LevelLightShadows,
};
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;

impl LevelLightShadows {
  /// Declares the faces this frame's `prepare` readied: each slot's arguments started from those a cull starts with,
  /// every face culled into its slot in one compute pass, then every face drawn into its square in one render pass, what
  /// the drawable `kinds` cast over those whose light casts from them.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      textures,
      statics,
      cull: params,
      ..
    }: ViewRecord<'a>,
    kinds: &[DrawableKindRef<'a>],
  ) {
    if self.due.is_empty() {
      return;
    }

    let scene: &StaticSceneHandles = &statics;
    let template: GraphBuffer = scene.args_template;
    let args: GraphBuffer = bindings.import_buffer(graph, "light shadow draw arguments", &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, "light shadow lists", self.lists.get_buffer());
    let Some(atlas) = &self.atlas else {
      return;
    };
    let atlas: GraphTexture = bindings.import_view(graph, "light shadow atlas", atlas);
    // The kinds this frame's record brought, which a face draws only those of.
    let present: DrawableKinds = kinds.iter().map(DrawableKindRef::get_kind).collect();
    // Each slot's cull into its range of the lists and the arguments, and its draws of that range.
    let slots: Vec<FaceSlot> = self
      .due
      .iter()
      .map(|due| {
        let (list_offset, args_offset): (u64, u64) =
          (due.slot as u64 * self.region, due.slot as u64 * self.args_stride);

        FaceSlot {
          due: FaceDue {
            casting: due.casting.intersect(present),
            ..*due
          },
          cull: scene.get_cull_parameters(
            StorageArrayMut::new_range(lists, list_offset, self.region),
            StorageArrayMut::new_range(args, args_offset, StaticCullScratch::ARGS_SIZE),
          ),
          layouts: scene.get_layout_draws(StorageArray::new_range(lists, list_offset, self.region)),
        }
      })
      .collect();

    graph
      .add_encoder_pass(FACE_PASSES[0])
      .buffer(template, GraphBufferAccess::CopySource)
      .buffer(args, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (template, args) = (context.get_buffer(template), context.get_buffer(args));

        for due in &self.due {
          context.get_encoder().copy_buffer_to_buffer(
            template,
            0,
            args,
            due.slot as u64 * self.args_stride,
            StaticCullScratch::ARGS_SIZE,
          );
        }
      });
    slots
      .iter()
      .fold(graph.add_compute_pass(FACE_PASSES[1]), |builder, slot| {
        builder.parameters(&slot.cull)
      })
      .record({
        let culls: Vec<FaceSlot> = slots.clone();

        move |context| {
          for FaceSlot { due, cull, .. } in &culls {
            passes
              .scene
              .cull
              .record_shadow(context, &self.views[due.slot], cull, params, ShadowCull::LightFace);
          }
        }
      });
    // What stands still of the faces whose static tiles are stale, into those tiles, before any tile is restored.
    let restore: Option<ShadowTileRestoreParameters> =
      self.statics.as_ref().map(|statics| ShadowTileRestoreParameters {
        static_atlas: bindings.import_view(&mut *graph, "light shadow static atlas", statics),
      });
    let statics: Vec<FaceSlot> = slots
      .iter()
      .filter(|slot| slot.due.draw == FaceDraw::Static)
      .copied()
      .collect();

    if let (Some(restore), false) = (restore, statics.is_empty()) {
      statics
        .iter()
        .fold(
          graph
            .add_raster_pass(FACE_PASSES[3])
            .depth(GraphDepthAttachment::new(restore.static_atlas, wgpu::LoadOp::Load))
            .buffer(args, GraphBufferAccess::Indirect),
          |builder, slot| StaticDraws::declare_layouts(builder, &slot.layouts),
        )
        .record(move |context| {
          let args: &wgpu::Buffer = context.get_buffer(args);

          for FaceSlot { due, layouts, .. } in &statics {
            passes.lighting.shadow.record_tile(
              context,
              due.tile.get_region(),
              None,
              StaticLayoutBindings::new(&self.views[due.slot], layouts, textures),
              IndirectArgs {
                buffer: args,
                offset: due.slot as u64 * self.args_stride,
              },
              ShadowCasters::Fixed,
            );
          }
        });
    }

    let is_restored: bool = slots.iter().any(|slot| slot.due.draw != FaceDraw::Whole);
    // The kinds any face this frame draws casters of, which the pass reads.
    let drawn: DrawableKinds = slots
      .iter()
      .fold(DrawableKinds::default(), |drawn, slot| drawn.union(slot.due.casting));
    let drawing: Vec<DrawableKindRef<'a>> = kinds
      .iter()
      .filter(|kind| drawn.has(kind.get_kind()))
      .copied()
      .collect();
    let builder = slots.iter().fold(
      graph
        .add_raster_pass(FACE_PASSES[2])
        .depth(GraphDepthAttachment::new(atlas, wgpu::LoadOp::Load))
        .buffer(args, GraphBufferAccess::Indirect),
      |builder, slot| StaticDraws::declare_layouts(builder, &slot.layouts),
    );
    let builder = match restore.filter(|_| is_restored) {
      Some(restore) => builder.parameters(&restore),
      None => builder,
    };
    let builder = drawing.iter().fold(builder, |builder, kind| kind.declare(builder));

    builder.record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      for FaceSlot { due, layouts, .. } in &slots {
        // A swaying light's face over what stands still, as kept; any other drawn whole.
        let (restored, casters): (Option<&ShadowTileRestoreParameters>, ShadowCasters) = match (due.draw, &restore) {
          (FaceDraw::Static | FaceDraw::Swaying, Some(restore)) => (Some(restore), ShadowCasters::Swaying),
          _ => (None, ShadowCasters::All),
        };

        passes.lighting.shadow.record_tile(
          context,
          due.tile.get_region(),
          restored,
          StaticLayoutBindings::new(&self.views[due.slot], layouts, textures),
          IndirectArgs {
            buffer: args,
            offset: due.slot as u64 * self.args_stride,
          },
          casters,
        );

        for kind in drawing.iter().filter(|kind| due.casting.has(kind.get_kind())) {
          kind.record_shadow(context, &self.views[due.slot]);
        }
      }
    });
  }
}
