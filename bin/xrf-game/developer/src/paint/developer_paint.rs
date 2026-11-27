use xrf_renderer::{
  RenderLayer, RenderLayerMesh, RenderLayerTexture, RenderLayerTextureId, RenderLayerTextures, RenderLayerVertex,
  RenderRect,
};

/// egui's output as the renderer's layer takes it: meshes in device pixels, and texture changes in a fixed order.
pub(crate) struct DeveloperPaint;

impl DeveloperPaint {
  /// The layer egui's tessellated `primitives` paint at `pixels_per_point`; paint callbacks are not drawn (none is made).
  pub(crate) fn create_layer(primitives: Vec<egui::ClippedPrimitive>, pixels_per_point: f32) -> RenderLayer {
    RenderLayer {
      meshes: primitives
        .into_iter()
        .filter_map(|clipped| match clipped.primitive {
          egui::epaint::Primitive::Mesh(mesh) => Some(RenderLayerMesh {
            clip: Self::create_clip(clipped.clip_rect, pixels_per_point),
            texture: Self::create_texture_id(mesh.texture_id),
            vertices: mesh
              .vertices
              .iter()
              .map(|vertex| RenderLayerVertex {
                position: [vertex.pos.x * pixels_per_point, vertex.pos.y * pixels_per_point],
                uv: [vertex.uv.x, vertex.uv.y],
                color: vertex.color.to_array(),
              })
              .collect(),
            indices: mesh.indices,
          }),
          egui::epaint::Primitive::Callback(_) => None,
        })
        .collect(),
    }
  }

  /// egui's texture changes, each texture's in the order egui made them, textures in id order so a run repeats (egui
  /// keeps them in hash maps, whose order does not).
  pub(crate) fn create_textures(mut delta: egui::TexturesDelta) -> RenderLayerTextures {
    let mut set: Vec<(egui::TextureId, egui::epaint::ImageDelta)> = std::mem::take(&mut delta.set)
      .into_iter()
      .flat_map(|(id, deltas)| deltas.into_iter().map(move |delta| (id, delta)))
      .collect();
    let mut free: Vec<RenderLayerTextureId> = std::mem::take(&mut delta.free)
      .into_iter()
      .map(Self::create_texture_id)
      .collect();

    // Stable, so one texture's patches keep the order egui gave them.
    set.sort_by_key(|(id, _)| Self::create_texture_id(*id));
    free.sort_unstable();

    RenderLayerTextures {
      set: set
        .into_iter()
        .map(|(id, delta)| Self::create_texture(id, &delta))
        .collect(),
      free,
    }
  }

  /// egui's managed textures on even ids, a user's on odd, so neither names the other's.
  fn create_texture_id(id: egui::TextureId) -> RenderLayerTextureId {
    match id {
      egui::TextureId::Managed(id) => RenderLayerTextureId(id << 1),
      egui::TextureId::User(id) => RenderLayerTextureId((id << 1) | 1),
    }
  }

  fn create_texture(id: egui::TextureId, delta: &egui::epaint::ImageDelta) -> RenderLayerTexture {
    let egui::ImageData::Color(image) = &delta.image;

    RenderLayerTexture {
      id: Self::create_texture_id(id),
      origin: delta.pos.map(|[x, y]| [x as u32, y as u32]),
      width: image.size[0] as u32,
      height: image.size[1] as u32,
      pixels: image.pixels.iter().flat_map(|pixel| pixel.to_array()).collect(),
      is_nearest: delta.options.magnification == egui::TextureFilter::Nearest,
    }
  }

  /// A clip rectangle in points as device pixels, rounded outward so nothing egui draws inside it is cut.
  fn create_clip(rect: egui::Rect, pixels_per_point: f32) -> RenderRect {
    let left: f32 = (rect.min.x * pixels_per_point).floor().max(0.0);
    let top: f32 = (rect.min.y * pixels_per_point).floor().max(0.0);
    let right: f32 = (rect.max.x * pixels_per_point).ceil().max(left);
    let bottom: f32 = (rect.max.y * pixels_per_point).ceil().max(top);

    RenderRect {
      x: left as i32,
      y: top as i32,
      width: (right - left) as u32,
      height: (bottom - top) as u32,
    }
  }
}
