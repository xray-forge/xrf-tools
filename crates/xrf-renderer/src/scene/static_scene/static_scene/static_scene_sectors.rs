use glam::{Vec3, Vec4};
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::{SectorGeometry, SectorPackage, SectorSurface};

use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::draw::slot::static_row::StaticRow;
use crate::scene::static_scene::draw::slot::static_slot::StaticSlot;
use crate::scene::static_scene::draw::slot::static_slot_draw::StaticSlotDraw;
use crate::scene::static_scene::draw::slot::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::draw::static_batch::StaticBatch;
use crate::scene::static_scene::draw::static_class::StaticClass;
use crate::scene::static_scene::draw::static_sight::StaticSight;
use crate::scene::static_scene::geometry::geometry_clusters::GeometryClusters;
use crate::scene::static_scene::geometry::static_geometry_base::StaticGeometryBase;
use crate::scene::static_scene::geometry::static_spans::StaticSpans;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::geometry::vertex::static_vertex_words::StaticVertexWords;
use crate::scene::static_scene::item::static_item_context::StaticItemContext;
use crate::scene::static_scene::item::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::item::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::item::static_sorted_run::StaticSortedRun;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::surface::build::static_surface_ask::StaticSurfaceAsk;
use crate::scene::static_scene::surface::static_surface_key::StaticSurfaceKey;
use crate::scene::static_scene::surface::static_surface_use::StaticSurfaceUse;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;

impl StaticScene {
  /// Puts one packed sector into the scene at the handle its poster allocated: its baked sections as single draws, its
  /// tree groups as rows.
  ///
  /// # Errors
  ///
  /// Returns an error for a handle that holds a sector already, or when a buffer would outgrow what the device allows;
  /// none of it stands in the scene, though the surfaces it put and the texture slots they asked for stay.
  pub(crate) fn add_sector(
    &mut self,
    context: &mut StaticItemContext<'_>,
    descriptors: &[XraySurfaceDescriptor],
    handle: ProxyHandle<StaticSectorProxy>,
    package: &SectorPackage,
  ) -> XrfResult {
    if self.sectors.is_held(handle) {
      return Err(XrfError::new_invalid_error(format!(
        "Sector posted at {handle:?}, which holds one already"
      )));
    }

    let mut writer: StaticSceneWriter = StaticSceneWriter::default();
    let mut swaying: Vec<(Vec4, f32)> = Vec::new();
    let description = &package.description;
    let buffer: &[u8] = &package.buffer;
    let sections: Vec<Vec4> = self.put_sections(&mut writer, context, descriptors, package);

    if let Some(impostors) = &description.impostors {
      self.put_impostors(&mut writer, impostors, buffer, context);
    }

    for mesh in 0..description.instances.len() as u32 {
      self.put_instances(&mut writer, &mut swaying, package, mesh, descriptors, context);
    }

    self
      .surfaces
      .put(context.device, context.queue, context.encoder, &writer);

    self.cull.check_capacities(context.device, &writer.capacities)?;

    let spans: StaticSpans = self.geometry.allocate(context.device, context.queue, &writer)?;

    writer.relocate(&spans);
    self.geometry.write(&spans, &writer);

    let rows: Vec<ProxyHandle<StaticRow>> = writer.rows.iter().map(|row| self.geometry.add_row(*row)).collect();

    let sphere: Vec4 = Self::bound_spheres(
      &sections
        .into_iter()
        .chain(writer.rows.iter().map(|row| row.sphere))
        .chain(
          writer
            .impostors
            .iter()
            .map(|impostor| impostor.sphere)
            .filter(|it| it.w >= 0.0),
        )
        .filter(|it| it.w >= 0.0)
        .collect::<Vec<Vec4>>(),
    );

    self.cull.add_capacities(&writer.capacities);
    self.sway_reach = swaying.iter().map(|(_, reach)| *reach).fold(self.sway_reach, f32::max);
    self.swaying.extend_from_slice(&swaying);
    self.note_change(sphere);

    self.sectors.insert(
      handle,
      StaticSectorProxy {
        sphere,
        sector: description.sector,
        spans,
        rows,
        capacities: writer.capacities,
        slot_infos: writer.slot_infos,
        cluster_slots: writer.cluster_slots,
        impostor_shaders: writer.impostor_shaders,
        trace_geometries: writer.trace_geometries,
        trace_groups: writer.trace_groups,
        sorted_runs: writer.sorted_runs,
        sights: writer.sights,
        swaying,
      },
    );

    Ok(())
  }

  pub(super) fn find_sector(&self, sector: u32) -> Option<&StaticSectorProxy> {
    self.sectors.as_slice().iter().find(|it| it.sector == sector)
  }

  /// Puts a sector's baked sections, each a single draw cut in clusters, answering the sphere each holds in the world.
  fn put_sections(
    &mut self,
    writer: &mut StaticSceneWriter,
    context: &mut StaticItemContext<'_>,
    descriptors: &[XraySurfaceDescriptor],
    package: &SectorPackage,
  ) -> Vec<Vec4> {
    let description = &package.description;
    let buffer: &[u8] = &package.buffer;
    let mut sections: Vec<Vec4> = Vec::new();

    if !description.sections.is_empty() && description.geometry.index_count > 0 {
      let base: StaticGeometryBase = Self::put_geometry(writer, StaticLayout::Baked, &description.geometry, buffer);
      let clusters: GeometryClusters<'_> = GeometryClusters::new(&description.geometry, buffer);

      for section in &description.sections {
        // A section drawing past the sector's indices would draw another's, or past the buffer.
        if section
          .draw
          .start
          .checked_add(section.draw.count)
          .is_none_or(|end| end > description.geometry.index_count)
        {
          log::warn!(
            "Sector {} draws {} indices from {}, past its {}; the section is left out",
            description.sector,
            section.draw.count,
            section.draw.start,
            description.geometry.index_count
          );

          continue;
        }

        let Some(StaticSurfaceUse { surface, class, passes }) = self.surfaces.resolve(
          writer,
          Self::level_surface(&section.surface, descriptors),
          context.textures,
          context.source,
        ) else {
          continue;
        };
        let batch: StaticBatch = StaticBatch {
          layout: StaticLayout::Baked,
          class,
        };
        let cut: Vec<(u32, u32, Vec4)> = clusters.cut(section.draw.start, section.draw.count);
        let cluster_spheres: Vec<Vec4> = cut.iter().map(|(_, _, sphere)| *sphere).collect();
        let sphere: Vec4 = Self::bound_spheres(&cluster_spheres);

        sections.push(sphere);
        let (slot, clusters): (u32, u32) = writer.put_slot(
          &base,
          cut,
          StaticSlotDraw {
            kind: StaticSlot::SINGLE,
            batch,
            surface,
            passes,
          },
          StaticSlotInfo {
            sector: description.sector,
            shader_id: section.surface.shader_id,
            mesh: None,
            first_place: 0,
          },
        );

        for list in batch.list_draw_lists(passes) {
          writer.capacities[list] += clusters;
        }

        if class == StaticClass::Sorted {
          writer.sorted_runs.push(StaticSortedRun {
            sphere,
            place: None,
            layout: StaticLayout::Baked,
            clusters: (writer.slots[slot as usize].first_cluster, clusters),
          });
        }

        let sight: StaticSight = StaticSight::of(class, passes);

        // By its clusters' spheres, which bound a wide lake or a building's windows far closer than the section's.
        if !sight.is_empty() {
          writer
            .sights
            .extend(cluster_spheres.into_iter().map(|cluster| (cluster, sight)));
        }

        if class.is_deferred() {
          writer.trace_geometries.push(StaticTraceGeometry {
            layout: StaticLayout::Baked,
            first_index: base.index_start + section.draw.start,
            index_count: section.draw.count,
            vertex_start: base.vertex_start,
            vertex_count: description.geometry.vertex_count,
            surface,
            is_cut_out: class == StaticClass::CutOut,
          });
        }
      }
    }

    sections
  }

  pub(super) fn put_geometry(
    writer: &mut StaticSceneWriter,
    layout: StaticLayout,
    geometry: &SectorGeometry,
    buffer: &[u8],
  ) -> StaticGeometryBase {
    writer.put_words(
      layout,
      &StaticVertexWords::pack_vertex_words(layout, geometry, buffer),
      &read_pods::<u32>(buffer, &geometry.indices),
    )
  }

  /// Puts a slot per band (`bands`, each a first index and a count) of an instanced group's geometry, each drawn as
  /// `draw` asks. Answers the band slots with their clusters, and the bands' bounding sphere in the mesh's own space.
  pub(super) fn put_bands(
    writer: &mut StaticSceneWriter,
    base: &StaticGeometryBase,
    clusters: &GeometryClusters<'_>,
    bands: &[(u32, u32)],
    draw: StaticSlotDraw,
    info: StaticSlotInfo,
  ) -> (Vec<(u32, u32)>, Vec4) {
    let band_slots: Vec<(u32, u32)> = bands
      .iter()
      .map(|range| writer.put_slot(base, clusters.cut(range.0, range.1), draw, info))
      .collect();
    let local: Vec4 = Self::bound_spheres(
      &writer.spheres[writer.spheres.len() - band_slots.iter().map(|it| it.1 as usize).sum::<usize>()..],
    );

    (band_slots, local)
  }

  /// A level's surface as `resolve_surface` takes one: its shader table entry, dressed by that entry's descriptor.
  pub(super) fn level_surface<'a>(
    surface: &'a SectorSurface,
    descriptors: &'a [XraySurfaceDescriptor],
  ) -> StaticSurfaceAsk<'a> {
    StaticSurfaceAsk {
      key: StaticSurfaceKey::Level(surface.shader_id),
      surface,
      descriptor: descriptors.get(surface.shader_id as usize),
    }
  }

  /// A sphere holding every sphere: centred on their box, as far out as the farthest edge.
  fn bound_spheres(spheres: &[Vec4]) -> Vec4 {
    let (low, high): (Vec3, Vec3) = spheres.iter().fold((Vec3::MAX, Vec3::MIN), |(low, high), sphere| {
      (
        low.min(sphere.truncate() - sphere.w),
        high.max(sphere.truncate() + sphere.w),
      )
    });
    let center: Vec3 = (low + high) * 0.5;
    let radius: f32 = spheres
      .iter()
      .map(|sphere| sphere.truncate().distance(center) + sphere.w)
      .fold(0.0, f32::max);

    if spheres.is_empty() {
      Vec4::new(0.0, 0.0, 0.0, -1.0)
    } else {
      center.extend(radius)
    }
  }
}
