use std::collections::{BTreeMap, HashMap};

use crate::contract::layer::render_layer::RenderLayer;
use crate::contract::layer::render_layer_texture::RenderLayerTexture;
use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;
use crate::contract::layer::render_layer_textures::RenderLayerTextures;
use crate::viewport::layer::layer_gpu::LayerGpu;
use crate::viewport::layer::layer_image::LayerImage;
use crate::viewport::layer::layer_region::LayerRegion;

/// A viewport's layer, drawn over its window. Its CPU side is what its host sent: the latest layer and its textures'
/// pixels, each change applied as it comes. Its GPU side follows what changed when a frame is drawn
/// (`LayerPass::prepare`), and is made again from the CPU side after the GPU is lost, since a host sends each texture's
/// pixels once.
#[derive(Default)]
pub(crate) struct ViewportLayer {
  /// The latest layer sent.
  pub layer: RenderLayer,
  pub images: HashMap<RenderLayerTextureId, LayerImage>,
  /// Whether `layer` changed since it was uploaded.
  pub is_layer_changed: bool,
  /// Textures changed since they were uploaded, with the region of each that changed; one freed since has no image.
  pub changed: BTreeMap<RenderLayerTextureId, LayerRegion>,
  pub gpu: LayerGpu,
}

impl ViewportLayer {
  /// Takes `layer` to be drawn from the next frame, in place of the one before.
  pub(crate) fn set(&mut self, layer: RenderLayer) {
    self.layer = layer;
    self.is_layer_changed = true;
  }

  /// Applies texture changes after those sent before: pixels set, then textures let go. A change that does not fit its
  /// texture is left out and logged.
  pub(crate) fn change_textures(&mut self, textures: RenderLayerTextures) {
    for set in textures.set {
      let id: RenderLayerTextureId = set.id;

      if let Err(reason) = self.set_pixels(set) {
        log::warn!("Layer texture {} not set: {reason}", id.0);
      }
    }

    for id in textures.free {
      self.images.remove(&id);
      self.changed.insert(id, LayerRegion::default());
    }
  }

  /// Lets its GPU side go with the GPU, for the next GPU's first frame to make again from its CPU side.
  pub(crate) fn release_gpu(&mut self) {
    self.gpu = LayerGpu::default();
    self.is_layer_changed = true;
    self.changed = self
      .images
      .iter()
      .map(|(id, image)| (*id, LayerRegion::new(0, 0, image.width, image.height)))
      .collect();
  }

  /// Whether its host sent anything to draw: a layer of a mesh or more, uploaded or not.
  pub(crate) fn is_sent(&self) -> bool {
    !self.layer.meshes.is_empty()
  }

  /// Whether it has anything to draw.
  pub(crate) fn is_drawn(&self) -> bool {
    !self.gpu.draws.is_empty()
  }

  /// Sets `set`'s pixels: a whole new image, or a region of one it holds.
  fn set_pixels(&mut self, set: RenderLayerTexture) -> Result<(), String> {
    let expected: usize = set.width as usize * set.height as usize * 4;

    if set.width == 0 || set.height == 0 || set.pixels.len() != expected {
      return Err(format!(
        "{} bytes sent for {}x{} pixels",
        set.pixels.len(),
        set.width,
        set.height
      ));
    }

    let Some([x, y]) = set.origin else {
      self.images.insert(
        set.id,
        LayerImage {
          width: set.width,
          height: set.height,
          pixels: set.pixels,
          is_nearest: set.is_nearest,
        },
      );
      self.mark(set.id, LayerRegion::new(0, 0, set.width, set.height));

      return Ok(());
    };
    let image: &mut LayerImage = self
      .images
      .get_mut(&set.id)
      .ok_or_else(|| String::from("a region is set before its texture is made"))?;

    if u64::from(x) + u64::from(set.width) > u64::from(image.width)
      || u64::from(y) + u64::from(set.height) > u64::from(image.height)
    {
      return Err(format!(
        "{}x{} pixels at {x},{y} pass the edge of its {}x{}",
        set.width, set.height, image.width, image.height
      ));
    }

    let row: usize = set.width as usize * 4;
    let stride: usize = image.width as usize * 4;

    for (index, source) in set.pixels.chunks_exact(row).enumerate() {
      let start: usize = (y as usize + index) * stride + x as usize * 4;

      image.pixels[start..start + row].copy_from_slice(source);
    }

    self.mark(set.id, LayerRegion::new(x, y, set.width, set.height));

    Ok(())
  }

  fn mark(&mut self, id: RenderLayerTextureId, region: LayerRegion) {
    self
      .changed
      .entry(id)
      .and_modify(|changed| *changed = changed.union(region))
      .or_insert(region);
  }
}
