use glam::{Mat4, UVec4, Vec4};
use xrf_error::{XrfError, XrfResult};
use xrf_renderer_core::ProxyHandle;
use xrf_visual::SectorImpostors;

use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::draw::slot::static_slot::StaticSlot;
use crate::scene::static_scene::draw::slot::static_slot_draw::StaticSlotDraw;
use crate::scene::static_scene::draw::slot::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::draw::static_batch::StaticBatch;
use crate::scene::static_scene::draw::static_class::StaticClass;
use crate::scene::static_scene::draw::static_sight::StaticSight;
use crate::scene::static_scene::geometry::static_geometry_base::StaticGeometryBase;
use crate::scene::static_scene::geometry::static_impostor::StaticImpostor;
use crate::scene::static_scene::geometry::static_spans::StaticSpans;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::item::model::static_model::StaticModel;
use crate::scene::static_scene::item::model::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::item::static_item_context::StaticItemContext;
use crate::scene::static_scene::item::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::static_scene::{MODEL_ROW_CLUSTERS, StaticScene};
use crate::scene::static_scene::surface::build::static_surface_ask::StaticSurfaceAsk;
use crate::scene::static_scene::surface::static_surface_key::StaticSurfaceKey;
use crate::scene::static_scene::surface::static_surface_use::StaticSurfaceUse;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;

impl StaticScene {
  /// Puts a spawned model into the scene at the handle its poster allocated: its geometry once and a slot a part, which
  /// each object standing as it adds a row of.
  ///
  /// # Errors
  ///
  /// Returns an error for a handle that holds a model already, or when a buffer would outgrow what the device allows;
  /// none of it stands in the scene, though the surfaces it put and the texture slots they asked for stay.
  pub(crate) fn add_model(
    &mut self,
    context: &mut StaticItemContext<'_>,
    handle: ProxyHandle<StaticModelProxy>,
    model: &StaticModel,
  ) -> XrfResult {
    if self.models.is_held(handle) {
      return Err(XrfError::new_invalid_error(format!(
        "Model posted at {handle:?}, which holds one already"
      )));
    }

    let mut writer: StaticSceneWriter = StaticSceneWriter::default();
    let base: StaticGeometryBase = writer.put_words(StaticLayout::Model, &model.words, &model.indices);
    let vertex_count: u32 = (model.words.len() / StaticLayout::STRIDE as usize) as u32;
    let mut slots: Vec<(u32, u32, StaticBatch, u32)> = Vec::new();
    // The composited parts' clusters, which each object lists to be drawn sorted.
    let mut sorted: Vec<(u32, u32)> = Vec::new();
    let mut sight: StaticSight = StaticSight::default();

    for part in &model.parts {
      let key: StaticSurfaceKey = StaticSurfaceKey::Model {
        shader: part.surface.shader_name.clone(),
        texture: part.surface.texture_name.clone(),
      };
      let Some(StaticSurfaceUse { surface, class, passes }) = self.surfaces.resolve(
        &mut writer,
        StaticSurfaceAsk {
          key,
          surface: &part.surface,
          descriptor: part.descriptor.as_ref(),
        },
        context.textures,
        context.source,
      ) else {
        continue;
      };
      let batch: StaticBatch = StaticBatch {
        layout: StaticLayout::Model,
        class,
      };

      if class.is_deferred() && model.skin.is_none() {
        let traced: StaticTraceGeometry = writer.put_densities(StaticTraceGeometry {
          layout: StaticLayout::Model,
          first_index: base.index_start + part.indices.0,
          index_count: part.indices.1,
          vertex_start: base.vertex_start,
          vertex_count,
          surface,
          is_cut_out: class == StaticClass::CutOut,
          first_density: 0,
        });

        writer.trace_geometries.push(traced);
      }

      // A row's clusters are culled one after another by one thread, so a large part is cut into several slots.
      for clusters in part.clusters.chunks(MODEL_ROW_CLUSTERS) {
        let first_cluster: u32 = writer.clusters.len() as u32;
        let (slot, count): (u32, u32) = writer.put_slot(
          &base,
          clusters.to_vec(),
          StaticSlotDraw {
            kind: StaticSlot::LISTED,
            batch,
            surface,
            passes,
          },
          StaticSlotInfo {
            sector: StaticSlotInfo::NO_SECTOR,
            shader_id: part.surface.shader_id,
            mesh: None,
            first_place: 0,
          },
        );

        if class == StaticClass::Composited {
          sorted.push((first_cluster, count));
        }

        sight |= StaticSight::of(class, passes);

        slots.push((slot, count, batch, passes));
      }
    }

    if let Some(skin) = &model.skin {
      writer.skins.extend_from_slice(&skin.links);
    }

    self
      .surfaces
      .put(context.device, context.queue, context.encoder, &writer);

    let spans: StaticSpans = self.geometry.allocate(context.device, context.queue, &writer)?;
    let first_slot: u32 = StaticSpans::get_offset(spans.slots.as_ref());
    let first_cluster: u32 = StaticSpans::get_offset(spans.clusters.as_ref());

    writer.relocate(&spans);
    self.geometry.write(&spans, &writer);
    self.note_change(Self::NOWHERE);

    self.models.insert(
      handle,
      StaticModelProxy {
        slots: slots
          .into_iter()
          .map(|(slot, count, batch, passes)| (first_slot + slot, count, batch, passes))
          .collect(),
        sorted: sorted
          .into_iter()
          .map(|(first, count)| (first_cluster + first, count))
          .collect(),
        sight,
        slot_infos: writer.slot_infos,
        cluster_slots: writer.cluster_slots,
        sphere: model.sphere,
        bounds: model.bounds,
        vertex_start: StaticSpans::get_offset(spans.words[StaticLayout::Model.get_index()].as_ref())
          + base.vertex_start,
        skin: model
          .skin
          .as_ref()
          .map(|skin| (StaticSpans::get_offset(spans.skins.as_ref()), skin.bones)),
        trace_geometries: writer.trace_geometries,
        spans,
      },
    );

    Ok(())
  }

  /// Takes a model out of the scene with every object standing as it, its ranges cleared and freed; false for one
  /// already gone.
  pub(crate) fn remove_model(&mut self, handle: ProxyHandle<StaticModelProxy>) -> bool {
    if !self.models.contains(handle) {
      return false;
    }

    for object in self.objects.list_standing(handle) {
      self.remove_object(object);
    }

    if let Some(model) = self.models.remove(handle) {
      self.geometry.release(model.spans);
    }

    self.note_change(Self::NOWHERE);

    true
  }

  /// Puts a sector's impostors in its own order, each with its run's surface; one no run dresses is never drawn.
  pub(super) fn put_impostors(
    &mut self,
    writer: &mut StaticSceneWriter,
    impostors: &SectorImpostors,
    buffer: &[u8],
    context: &mut StaticItemContext<'_>,
  ) {
    let spheres: Vec<Vec4> = read_pods(buffer, &impostors.spheres);
    let factors: Vec<f32> = read_pods(buffer, &impostors.factors);
    let corners: Vec<[f32; SectorImpostors::FLOATS_PER_CORNER]> = read_pods(buffer, &impostors.corners);
    let normals: Vec<Vec4> = read_pods(buffer, &impostors.normals);
    let count: usize = spheres
      .len()
      .min(factors.len())
      .min(corners.len() / StaticImpostor::CORNERS)
      .min(normals.len() / StaticImpostor::FACETS);
    let mut dressed: Vec<Option<(u32, u16)>> = vec![None; count];

    for group in &impostors.groups {
      let row: u32 = self
        .surfaces
        .resolve_impostor(writer, &group.surface, context.textures, context.source);

      for index in (group.start as usize..).take(group.count as usize) {
        if let Some(it) = dressed.get_mut(index) {
          *it = Some((row, group.surface.shader_id));
        }
      }
    }

    for (index, dressing) in dressed.iter().enumerate() {
      let mut sphere: Vec4 = spheres[index];

      if dressing.is_none() {
        sphere.w = -1.0;
      }

      let mut facets: [Vec4; StaticImpostor::FACETS] = [Vec4::ZERO; StaticImpostor::FACETS];

      facets.copy_from_slice(&normals[index * StaticImpostor::FACETS..][..StaticImpostor::FACETS]);
      writer.impostors.push(StaticImpostor {
        sphere,
        normals: facets,
        factor: factors[index],
        surface: dressing.map_or(0, |it| it.0),
        _pad: [0; 2],
      });

      for corner in &corners[index * StaticImpostor::CORNERS..][..StaticImpostor::CORNERS] {
        // Position, atlas coordinate, then the hemisphere and sun terms.
        writer
          .corners
          .push(Vec4::new(corner[0], corner[1], corner[2], corner[5]));
        writer.corners.push(Vec4::new(corner[3], corner[4], corner[6], 0.0));
      }

      writer.impostor_shaders.push(dressing.map_or(0, |it| it.1));
    }
  }

  /// The largest scale a matrix stands anything at, which a sphere it moves grows by.
  pub(super) fn get_scale(matrix: &Mat4) -> f32 {
    [matrix.x_axis, matrix.y_axis, matrix.z_axis]
      .iter()
      .map(|axis| axis.truncate().length())
      .fold(0.0, f32::max)
  }

  /// A spawned object's lighting as a place holds it: each cube face a byte, `+x +y +z -x` in `x`, `-y -z` in `y`, the
  /// sky share's bits in `z`, and `w` marking it.
  pub(super) fn pack_cube((faces, sky): ([f32; 6], f32)) -> UVec4 {
    let bytes: [u32; 6] = faces.map(|face| (face.clamp(0.0, 1.0) * 255.0).round() as u32);

    UVec4::new(
      bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24),
      bytes[4] | (bytes[5] << 8),
      sky.to_bits(),
      1,
    )
  }
}
