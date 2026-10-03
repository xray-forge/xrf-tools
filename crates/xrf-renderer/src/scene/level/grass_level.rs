use std::sync::Arc;

use glam::Vec4;
use wgpu::util::DeviceExt;
use xrf_material::XraySurfaceDraw;
use xrf_visual::{DetailsDescription, DetailsModel, VisualSection};

use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_details::RenderLevelDetails;
use crate::pass::grass_model_record::GrassModelRecord;
use crate::pass::grass_pass::{GRASS_DRAW_BYTES, GrassPass};
use crate::scene::level::grass_dither::create_grass_dither;
use crate::scene::section_bytes::read_pods;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// A level's grass on the GPU as it is planted from, made once: the slot grid, the planted slots and their triangle
/// bins, the dither, the models with their vertices, and what each frame counts and draws them into.
pub struct GrassLevel {
  /// The grid's size in slots, and the world slot its first cell stands for, negated.
  pub grid: [i32; 4],
  pub model_count: u32,
  /// Metres the largest tuft reaches past the ground it stands on, at a height of one.
  pub tuft_reach: f32,
  /// The texture slots its models sample.
  pub texture_slots: Vec<u32>,
  pub models: wgpu::Buffer,
  pub positions: wgpu::Buffer,
  pub uvs: wgpu::Buffer,
  pub indices: wgpu::Buffer,
  pub args: wgpu::Buffer,
  pub bind_group: wgpu::BindGroup,
}

impl GrassLevel {
  pub fn new(
    device: &wgpu::Device,
    pass: &GrassPass,
    uniform: &wgpu::Buffer,
    details: &RenderLevelDetails,
    (textures, source): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
  ) -> Self {
    let description: &DetailsDescription = &details.package.description;
    let buffer: &[u8] = &details.package.buffer;
    let storage = |label: &str, bytes: &[u8], usage: wgpu::BufferUsages| -> wgpu::Buffer {
      // A binding cannot be empty: an empty array is bound as one element.
      let padded: Vec<u8> = if bytes.is_empty() { vec![0; 4] } else { bytes.to_vec() };

      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &padded,
        usage,
      })
    };
    let read = wgpu::BufferUsages::STORAGE;
    let section = |section: &VisualSection| -> &[u8] {
      let start: usize = (section.byte_offset as usize).min(buffer.len());

      &buffer[start..(start + section.byte_length as usize).min(buffer.len())]
    };
    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut records: Vec<GrassModelRecord> = Vec::new();
    let mut texture_slots: Vec<u32> = Vec::new();

    for (index, model) in description.models.iter().enumerate() {
      let base: u32 = (positions.len() / 3) as u32;
      let first_index: u32 = indices.len() as u32;
      let model_indices: Vec<u16> = read_pods(buffer, &model.indices);
      let texture: u32 = textures.request(&model.texture, TextureRole::Base, source);
      let reference: u8 = match details.surfaces.get(index).map(|it| it.draw) {
        Some(XraySurfaceDraw::AlphaTested { reference }) => reference,
        _ => XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
      };

      positions.extend(read_pods::<f32>(buffer, &model.positions));
      uvs.extend(read_pods::<f32>(buffer, &model.uvs));
      indices.extend(model_indices.iter().map(|it| u32::from(*it) + base));
      texture_slots.push(texture);
      records.push(to_record(
        model,
        texture,
        reference,
        first_index,
        model_indices.len() as u32,
      ));
    }

    let model_count: u32 = records.len() as u32;
    let records_bytes: &[u8] = bytemuck::cast_slice(&records);
    let models: wgpu::Buffer = storage("grass models", records_bytes, read);
    let grid: wgpu::Buffer = storage("grass grid", section(&description.grid), read);
    let slots: wgpu::Buffer = storage("grass slots", section(&description.slots), read);
    let bins: wgpu::Buffer = storage("grass bins", section(&description.bins), read);
    let triangles: wgpu::Buffer = storage("grass triangles", section(&description.triangles), read);
    let dither: wgpu::Buffer = storage("grass dither", bytemuck::cast_slice(&create_grass_dither()), read);
    let counted = wgpu::BufferUsages::STORAGE;
    let counts: wgpu::Buffer = storage("grass counts", &vec![0; (model_count as usize + 1) * 4], counted);
    let cursors: wgpu::Buffer = storage("grass cursors", &vec![0; model_count.max(1) as usize * 4], counted);
    let args: wgpu::Buffer = storage(
      "grass draw arguments",
      &vec![0; (u64::from(model_count.max(1)) * GRASS_DRAW_BYTES) as usize],
      wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
    );
    let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("grass level"),
      layout: pass.get_level_layout(),
      entries: &[
        uniform, &grid, &slots, &bins, &triangles, &dither, &models, &counts, &cursors, &args,
      ]
      .iter()
      .enumerate()
      .map(|(binding, buffer)| wgpu::BindGroupEntry {
        binding: binding as u32,
        resource: buffer.as_entire_binding(),
      })
      .collect::<Vec<_>>(),
    });
    let vertex = wgpu::BufferUsages::VERTEX;

    Self {
      grid: [
        description.size_x as i32,
        description.size_z as i32,
        description.offset_x,
        description.offset_z,
      ],
      model_count,
      tuft_reach: description
        .models
        .iter()
        .map(|model| model.max_scale * (model.radius + model.height))
        .fold(0.0, f32::max),
      texture_slots,
      positions: storage("grass positions", bytemuck::cast_slice(&positions), vertex),
      uvs: storage("grass coordinates", bytemuck::cast_slice(&uvs), vertex),
      indices: storage(
        "grass indices",
        bytemuck::cast_slice(&indices),
        wgpu::BufferUsages::INDEX,
      ),
      models,
      args,
      bind_group,
    }
  }
}

fn to_record(
  model: &DetailsModel,
  texture: u32,
  reference: u8,
  first_index: u32,
  index_count: u32,
) -> GrassModelRecord {
  GrassModelRecord {
    shape: Vec4::new(model.min_scale, model.max_scale, model.radius, model.height),
    is_waving: if model.is_waving { 1.0 } else { 0.0 },
    texture,
    alpha_reference: f32::from(reference) / 255.0,
    index_count,
    first_index,
    pad: [0; 3],
  }
}
