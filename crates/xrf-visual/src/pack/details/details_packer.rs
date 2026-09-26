use std::collections::HashMap;
use std::ops::RangeInclusive;

use byteorder::ByteOrder;
use xrf_level::{DETAIL_SLOT_METERS, DetailModel, LevelCformFace, LevelCformGeometry, LevelDetailsFile};
use xrf_math::Vector3d;

use crate::data::details::details_description::DetailsDescription;
use crate::data::details::details_model::DetailsModel;
use crate::pack::details::details_package::DetailsPackage;
use crate::pack::details::details_slot_box::DetailsSlotBox;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_vector, reverse_triangle_winding};

/// `u32` words a packed slot record takes.
const SLOT_WORDS: usize = 6;

/// Metres a triangle's footprint is widened by on each side before it is walked over the grid, covering the room the
/// slot boxes grow by.
const FOOTPRINT_MARGIN: f32 = 0.01;

/// Packs a level's grass: its detail models, and each planted slot with the collision triangles its planting is cast
/// down onto, binned once by the box query `CDetailManager::cache_Decompress` makes for it.
pub struct DetailsPacker<'a> {
  details: &'a LevelDetailsFile,
  collision: &'a LevelCformGeometry,
  is_passable: &'a dyn Fn(u16) -> bool,
}

impl<'a> DetailsPacker<'a> {
  /// # Arguments
  ///
  /// * `is_passable` - Whether a game material, by its id, lets a planting fall through it (`flPassable`).
  pub fn new(
    details: &'a LevelDetailsFile,
    collision: &'a LevelCformGeometry,
    is_passable: &'a dyn Fn(u16) -> bool,
  ) -> Self {
    Self {
      details,
      collision,
      is_passable,
    }
  }

  /// The grass as the renderer plants it: every model, and each planted slot with the ground its planting falls on.
  pub fn pack<T: ByteOrder>(&self) -> DetailsPackage {
    let header = &self.details.header;
    let (size_x, size_z) = (header.size_x as i64, header.size_z as i64);
    let bins: Vec<Vec<u32>> = self.bin::<T>();

    let mut triangles: Vec<f32> = Vec::new();
    let mut compacted: HashMap<u32, u32> = HashMap::new();
    let mut entries: Vec<u32> = Vec::new();
    let mut records: Vec<u32> = Vec::new();
    let mut grid: Vec<u32> = vec![0; bins.len()];

    for (cell, bin) in bins.iter().enumerate().filter(|(_, bin)| !bin.is_empty()) {
      let start: u32 = entries.len() as u32;

      for face in bin {
        let index: u32 = *compacted.entry(*face).or_insert_with(|| {
          for corner in self.triangle(*face) {
            triangles.extend_from_slice(&corner);
          }

          (triangles.len() / 9 - 1) as u32
        });

        entries.push(index);
      }

      // A cell holding a bin is a cell `bin` found a stored slot for.
      if let Some(stored) = self.details.get_stored_slot(cell) {
        records.extend(stored.as_chunks::<4>().0.iter().map(|word| T::read_u32(word)));
      }

      records.extend([start, bin.len() as u32]);
      grid[cell] = (records.len() / SLOT_WORDS) as u32;
    }

    debug_assert_eq!(grid.len() as i64, size_x * size_z, "a grid holds a cell a slot");

    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
    let models: Vec<DetailsModel> = self
      .details
      .objects
      .iter()
      .map(|model| Self::pack_model(model, &mut builder))
      .collect();
    let grid_section = builder.push_u32_section(&grid);
    let slots_section = builder.push_u32_section(&records);
    let bins_section = builder.push_u32_section(&entries);
    let triangles_section = builder.push_f32_section(&triangles);

    DetailsPackage {
      description: DetailsDescription {
        bin_length: entries.len() as u32,
        bins: bins_section,
        buffer_length: builder.length(),
        grid: grid_section,
        models,
        offset_x: header.offset_x,
        offset_z: header.offset_z,
        size_x: header.size_x,
        size_z: header.size_z,
        slot_count: (records.len() / SLOT_WORDS) as u32,
        slots: slots_section,
        triangle_count: (triangles.len() / 9) as u32,
        triangles: triangles_section,
      },
      buffer: builder.into_buffer(),
    }
  }

  /// Each cell's triangles, found by walking every solid triangle over the cells its footprint covers rather than
  /// every cell over every triangle.
  fn bin<T: ByteOrder>(&self) -> Vec<Vec<u32>> {
    let header = &self.details.header;
    let size_x: i64 = i64::from(header.size_x);
    let mut bins: Vec<Vec<u32>> = vec![Vec::new(); self.details.get_slots_count() as usize];

    for (index, face) in self.collision.faces.iter().enumerate() {
      if (self.is_passable)(face.material) {
        continue;
      }

      let triangle: [[f32; 3]; 3] = self.triangle(index as u32);

      if !triangle.as_flattened().iter().all(|value| value.is_finite()) {
        continue;
      }

      let columns: RangeInclusive<i64> =
        Self::to_cell_span(triangle.map(|corner| corner[0]), header.offset_x, header.size_x);
      let rows: RangeInclusive<i64> =
        Self::to_cell_span(triangle.map(|corner| corner[2]), header.offset_z, header.size_z);

      for z in rows {
        for x in columns.clone() {
          let cell: usize = (z * size_x + x) as usize;
          let Some(slot) = self.details.get_slot::<T>(cell) else {
            continue;
          };
          let (world_x, world_z) = self.to_world_slot(x, z);

          if slot.is_planted()
            && DetailsSlotBox::of(world_x, world_z, slot.base_height, slot.height).overlaps(&triangle)
          {
            bins[cell].push(index as u32);
          }
        }
      }
    }

    bins
  }

  /// A cell's world slot, `CDetailManager::QueryDB` read backwards.
  fn to_world_slot(&self, x: i64, z: i64) -> (i32, i32) {
    let header = &self.details.header;

    ((x - header.offset_x as i64) as i32, (z - header.offset_z as i64) as i32)
  }

  /// The grid cells along one axis a span of finite values touches, with the room the slot boxes grow by, cut to the
  /// grid: empty for a span wholly outside it.
  fn to_cell_span(values: [f32; 3], offset: i32, size: u32) -> RangeInclusive<i64> {
    let minimum: f32 = values[0].min(values[1]).min(values[2]);
    let maximum: f32 = values[0].max(values[1]).max(values[2]);
    // Saturating casts, so a span reaching past `i64` still cuts to the grid.
    let to_cell = |value: f32| ((value / DETAIL_SLOT_METERS).floor() as i64).saturating_add(i64::from(offset));

    to_cell(minimum - FOOTPRINT_MARGIN).max(0)..=to_cell(maximum + FOOTPRINT_MARGIN).min(i64::from(size) - 1)
  }

  fn triangle(&self, face: u32) -> [[f32; 3]; 3] {
    let face: &LevelCformFace = &self.collision.faces[face as usize];

    self
      .collision
      .get_triangle(face)
      .map(|corner| [corner.x, corner.y, corner.z])
  }

  fn pack_model(model: &DetailModel, builder: &mut VisualBufferBuilder) -> DetailsModel {
    let (minimum, maximum) = model
      .get_bounds()
      .unwrap_or((Vector3d::new(0.0, 0.0, 0.0), Vector3d::new(0.0, 0.0, 0.0)));
    let half: [f32; 3] = [
      (maximum.x - minimum.x) * 0.5,
      (maximum.y - minimum.y) * 0.5,
      (maximum.z - minimum.z) * 0.5,
    ];

    let mut positions: Vec<f32> = Vec::with_capacity(model.vertices.len() * 3);
    let mut uvs: Vec<f32> = Vec::with_capacity(model.vertices.len() * 2);

    for vertex in &model.vertices {
      let converted: Vector3d = convert_vector(&vertex.position);

      positions.extend([converted.x, converted.y, converted.z]);
      uvs.extend([vertex.u, vertex.v]);
    }

    let mut indices: Vec<u16> = model.indices.clone();

    reverse_triangle_winding(&mut indices);

    DetailsModel {
      height: maximum.y - minimum.y,
      index_count: indices.len() as u32,
      indices: builder.push_u16_section(&indices),
      is_waving: model.is_waving(),
      max_scale: model.max_scale,
      min_scale: model.min_scale,
      positions: builder.push_f32_section(&positions),
      // `Fbox::getsphere`: from the box's centre to its corner.
      radius: (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt(),
      shader: model.shader.clone(),
      texture: model.texture.clone(),
      uvs: builder.push_f32_section(&uvs),
      vertex_count: model.vertices.len() as u32,
    }
  }
}
