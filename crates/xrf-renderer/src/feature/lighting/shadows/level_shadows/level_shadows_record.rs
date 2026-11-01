use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDeclaration, GraphDepthAttachment, GraphTexture,
  StorageArray, StorageArrayMut,
};

use crate::feature::lighting::shadows::cascade_target::CascadeTarget;
use crate::feature::lighting::shadows::level_shadows::{
  BAND_PASSES, CascadeDraw, KEPT_PASSES, LevelShadows, STATIC_PASSES, SWAY_PASSES, WHOLE_PASSES,
};
use crate::feature::lighting::shadows::shadow_casters::ShadowCasters;
use crate::feature::lighting::shadows::shadow_cut::ShadowCut;
use crate::feature::lighting::shadows::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
use crate::feature::scene::drawable::casters::drawable_kind_ref::DrawableKindRef;
use crate::feature::scene::statics::cull::shadow_cull::ShadowCull;
use crate::feature::scene::statics::cull::static_cull_parameters::StaticCullParameters;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::scene::statics::draw::static_draws::StaticDraws;
use crate::feature::scene::statics::draw::static_layout_bindings::StaticLayoutBindings;
use crate::frame::targets::texture_copy::add_layer_copy;
use crate::frame::targets::texture_layer::TextureLayer;
use crate::pass::build::indirect_args::IndirectArgs;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

/// A cut's cull and draw as a frame's graph knows them: its draws' arguments, its cull's parameters and what it draws,
/// layout by layout.
#[derive(Clone, Copy)]
struct CutHandles<'a> {
  cut: &'a ShadowCut,
  args: GraphBuffer,
  cull: StaticCullParameters,
  layouts: [StaticDrawParameters; StaticLayout::COUNT],
}

/// What the cascades draw into as a frame's graph knows it: the maps the sun samples, and the static and kept layers
/// where they are made.
#[derive(Clone, Copy)]
struct CascadeTextures {
  maps: GraphTexture,
  statics: Option<GraphTexture>,
  kept: Option<GraphTexture>,
}

impl LevelShadows {
  /// Declares the cascades due this frame and the bands drawn ahead: every one culled into its own lists in one compute
  /// pass, then each cascade drawn into its layer of the maps, whole or over its static layer's copy with what of the
  /// drawable `kinds` moves every frame over those it lands in, and each band into its cascade's hidden static layer.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    kinds: &[DrawableKindRef<'a>],
  ) {
    let ViewRecord {
      passes,
      statics: scene,
      cull: params,
      lit,
      ..
    } = record;

    if self.due.is_empty() && self.bands.is_empty() {
      return;
    }

    let textures: CascadeTextures = CascadeTextures {
      maps: lit.shadow_maps,
      statics: self
        .statics
        .as_ref()
        .map(|statics| bindings.import_view(&mut *graph, "sun static shadow maps", &statics.view)),
      kept: self
        .kept
        .as_ref()
        .map(|kept| bindings.import_view(&mut *graph, "sun kept shadow maps", &kept.view)),
    };
    let mut import = |cut: &'a ShadowCut| -> CutHandles<'a> {
      let args: GraphBuffer = bindings.import_buffer(&mut *graph, "sun shadow draw arguments", &cut.args);
      let lists: GraphBuffer = bindings.import_buffer(&mut *graph, "sun shadow lists", cut.get_lists());

      CutHandles {
        cut,
        args,
        cull: scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args)),
        layouts: scene.get_layout_draws(StorageArray::new(lists)),
      }
    };
    let due: Vec<(usize, CascadeDraw, CutHandles<'a>)> = self
      .due
      .iter()
      .map(|&(index, draw)| (index, draw, import(&self.cascades[index].cut)))
      .collect();
    let bands: Vec<(usize, u32, u32, CutHandles<'a>)> = self
      .bands
      .iter()
      .filter_map(|&(index, band, layer)| Some((index, band, layer, import(self.cascades[index].band.as_ref()?))))
      .collect();
    // A kept layer's copy draws no caster, so its cascade needs no lists; one whose trees go into its kept layer keeps
    // them only where their shadows can fall in the camera's view, the layer drawn again once the camera turns away.
    let culls: Vec<(&'a ViewBinding, StaticCullParameters, ShadowCull)> = due
      .iter()
      .filter(|(_, draw, _)| *draw != CascadeDraw::Kept)
      .map(|(index, _, handles)| {
        let kind: ShadowCull = if self.is_kept_draw(*index) {
          ShadowCull::CascadeReceived
        } else {
          ShadowCull::Cascade
        };

        (&handles.cut.view, handles.cull, kind)
      })
      .chain(
        bands
          .iter()
          .map(|(.., handles)| (&handles.cut.view, handles.cull, ShadowCull::Cascade)),
      )
      .collect();

    culls
      .iter()
      .fold(graph.add_compute_pass("sun shadow cull"), |builder, (_, cull, _)| {
        builder.parameters(cull)
      })
      .record(move |context| {
        for (view, cull, kind) in &culls {
          passes.scene.cull.record_shadow(context, view, cull, params, *kind);
        }
      });

    for (index, draw, handles) in due {
      self.add_cascade(
        (&mut *graph, &mut *bindings),
        record,
        kinds,
        textures,
        (index, draw, handles),
      );
    }

    let (Some(statics), Some(resolution)) = (textures.statics, self.maps.as_ref().map(|maps| maps.resolution)) else {
      return;
    };

    for (index, band, layer, CutHandles { cut, args, layouts, .. }) in bands {
      StaticDraws::declare_layouts(
        graph
          .add_raster_pass(BAND_PASSES[index][0])
          .depth(GraphDepthAttachment::new(statics, wgpu::LoadOp::Load).with_array_layer(layer))
          .buffer(args, GraphBufferAccess::Indirect),
        &layouts,
      )
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        passes.lighting.shadow.record_tile(
          context,
          Self::get_band_region(band, resolution),
          None,
          StaticLayoutBindings::new(&cut.view, &layouts, record.textures),
          IndirectArgs::of(args),
          ShadowCasters::Fixed,
        );
      });
    }
  }

  /// Declares cascade `index` drawn as `draw` with its cut's `handles`: its layer of the maps copied from its kept
  /// layer, a band of its static layer restored, its static layer copied in under the trees, or drawn whole; then what
  /// of the drawable `kinds` moves every frame and lands in it, over a static or kept layer's copy.
  fn add_cascade<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    ViewRecord { passes, textures, .. }: ViewRecord<'a>,
    kinds: &[DrawableKindRef<'a>],
    CascadeTextures { maps, statics, kept }: CascadeTextures,
    (index, draw, CutHandles { cut, args, layouts, .. }): (usize, CascadeDraw, CutHandles<'a>),
  ) {
    let layer: u32 = index as u32;
    let static_layer: u32 = self.cascades[index].static_layer;
    let draw_casters = |graph: &mut FrameGraph<'a>,
                        name: &'static str,
                        (target, layer, load): (GraphTexture, u32, wgpu::LoadOp<f32>),
                        casters: ShadowCasters| {
      StaticDraws::declare_layouts(
        graph
          .add_raster_pass(name)
          .depth(GraphDepthAttachment::new(target, load).with_array_layer(layer))
          .buffer(args, GraphBufferAccess::Indirect),
        &layouts,
      )
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        passes.lighting.shadow.record_some(
          context,
          StaticLayoutBindings::new(&cut.view, &layouts, textures),
          args,
          casters,
        );
      });
    };
    // What moves every frame and lands in the cascade, drawn over its map by each kind.
    let add_moving = |graph: &mut FrameGraph<'a>| {
      for landed in self.per_frame.iter().filter(|landed| landed.cascades[index]) {
        for kind in kinds.iter().filter(|kind| kind.get_kind() == landed.casters.kind) {
          kind.add_sun_pass(
            graph,
            CascadeTarget {
              name: landed.casters.passes[index],
              maps,
              layer,
              view: &cut.view,
            },
          );
        }
      }
    };

    match (draw, statics, kept) {
      (CascadeDraw::Kept, _, Some(kept)) => {
        add_layer_copy(
          &mut *graph,
          KEPT_PASSES[index][0],
          TextureLayer { texture: kept, layer },
          TextureLayer { texture: maps, layer },
        );
        add_moving(&mut *graph);
      }
      (CascadeDraw::Banded(band), Some(_), _) => {
        let Some(static_views) = self.statics.as_ref() else {
          return;
        };
        let region: [u32; 4] = Self::get_band_region(band, self.maps.as_ref().map_or(1, |maps| maps.resolution));
        // Its static layer's rows restored by a draw, a depth target taking no partial copy, the trees over them.
        let restore: ShadowTileRestoreParameters = ShadowTileRestoreParameters {
          static_atlas: bindings.import_view(
            &mut *graph,
            "sun static shadow layer",
            &static_views.layers[static_layer as usize],
          ),
        };

        StaticDraws::declare_layouts(
          graph
            .add_raster_pass(SWAY_PASSES[index][0])
            .depth(GraphDepthAttachment::new(maps, wgpu::LoadOp::Load).with_array_layer(layer))
            .buffer(args, GraphBufferAccess::Indirect)
            .parameters(&restore),
          &layouts,
        )
        .record(move |context| {
          let args: &wgpu::Buffer = context.get_buffer(args);

          passes.lighting.shadow.record_tile(
            context,
            region,
            Some(&restore),
            StaticLayoutBindings::new(&cut.view, &layouts, textures),
            IndirectArgs::of(args),
            ShadowCasters::Swaying,
          );
        });
      }
      (CascadeDraw::Static | CascadeDraw::Swaying, Some(statics), kept) => {
        let [static_pass, copy_pass, sway_pass] = STATIC_PASSES[index];
        // Where anything moving every frame lands, the trees go into the kept layer, which later frames copy in under
        // what moves alone.
        let kept: Option<GraphTexture> = kept.filter(|_| self.has_moving(index));
        let target: GraphTexture = kept.unwrap_or(maps);

        if draw == CascadeDraw::Static {
          draw_casters(
            &mut *graph,
            static_pass,
            (statics, static_layer, wgpu::LoadOp::Clear(0.0)),
            ShadowCasters::Fixed,
          );
        }

        add_layer_copy(
          &mut *graph,
          copy_pass,
          TextureLayer {
            texture: statics,
            layer: static_layer,
          },
          TextureLayer { texture: target, layer },
        );
        draw_casters(
          &mut *graph,
          sway_pass,
          (target, layer, wgpu::LoadOp::Load),
          ShadowCasters::Swaying,
        );

        if let Some(kept) = kept {
          add_layer_copy(
            &mut *graph,
            KEPT_PASSES[index][0],
            TextureLayer { texture: kept, layer },
            TextureLayer { texture: maps, layer },
          );
        }

        add_moving(&mut *graph);
      }
      _ => draw_casters(
        &mut *graph,
        WHOLE_PASSES[index][0],
        (maps, layer, wgpu::LoadOp::Clear(0.0)),
        ShadowCasters::All,
      ),
    }
  }
}
