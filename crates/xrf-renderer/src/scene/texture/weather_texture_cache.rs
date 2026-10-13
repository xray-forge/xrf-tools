use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::host::render_asset_source::{RenderAssetSource, get_texture_name};
use crate::lighting::render_sky::RenderSky;
use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// Frames a texture no viewport asks for is kept, so the skies of a weather faded away a moment ago come back at once.
const KEPT_FRAMES: u64 = 600;

/// A half in every channel, which ripples nothing, for a volume until its file is up.
const PLACEHOLDER_VOLUME: [u8; 4] = [128, 128, 128, 128];

/// The mean of `default_clear`'s noon irradiance, which a sky cube shows until its file is up.
const PLACEHOLDER_SKY: [u8; 4] = [128, 130, 140, 255];

/// What a texture load came to, sent from a loader thread.
type SkyTextureLoad = (String, Result<Option<DecodedTexture>, String>);

/// One weather texture: its view once up, whether its load settled, and the source it was read from.
struct SkyTextureEntry {
  kind: WeatherTextureKind,
  source: Arc<dyn RenderAssetSource>,
  view: Option<wgpu::TextureView>,
  /// Whether it is up, or given up on for good.
  is_settled: bool,
  /// The frame it was last asked for.
  used: u64,
}

/// The weather's textures, skies and their irradiance cubes, clouds, rain and what it wets with, by reference: loaded the first time a viewport
/// asks for one, and let go once none has asked for a while.
pub struct WeatherTextureCache {
  entries: HashMap<String, SkyTextureEntry>,
  sender: Sender<SkyTextureLoad>,
  receiver: LoaderReceiver<SkyTextureLoad>,
  placeholder_cube: wgpu::TextureView,
  placeholder_flat: wgpu::TextureView,
  placeholder_volume: wgpu::TextureView,
  frame: u64,
  workers: RenderWorkers,
}

impl WeatherTextureCache {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();

    Self {
      entries: HashMap::new(),
      sender,
      receiver,
      placeholder_cube: Self::create_solid(
        device,
        queue,
        PLACEHOLDER_SKY,
        DecodedTexture::CUBE_FACES,
        wgpu::TextureViewDimension::Cube,
      ),
      placeholder_flat: Self::create_solid(device, queue, [0, 0, 0, 0], 1, wgpu::TextureViewDimension::D2),
      placeholder_volume: Self::create_solid(
        device,
        queue,
        PLACEHOLDER_VOLUME,
        1,
        wgpu::TextureViewDimension::D2Array,
      ),
      frame: 0,
      workers: workers.clone(),
    }
  }

  /// Asks for a texture, loading it on a loader thread the first time; answers whether it is up or given up on.
  pub fn request(&mut self, reference: &str, kind: WeatherTextureKind, source: &Arc<dyn RenderAssetSource>) -> bool {
    if let Some(entry) = self.entries.get_mut(reference) {
      entry.used = self.frame;

      return entry.is_settled;
    }

    self.entries.insert(
      reference.to_owned(),
      SkyTextureEntry {
        kind,
        source: Arc::clone(source),
        view: None,
        is_settled: false,
        used: self.frame,
      },
    );
    self.load(reference, source);

    false
  }

  /// Loads again every texture of these file names (lowercase, without folder or extension), its source first
  /// forgetting where it resolved: its file changed. Each keeps what it shows until it lands again.
  pub fn reload(&mut self, names: &[String]) {
    let reloaded: Vec<(String, Arc<dyn RenderAssetSource>)> = self
      .entries
      .iter()
      .filter(|(reference, _)| names.contains(&get_texture_name(reference)))
      .map(|(reference, entry)| (reference.clone(), Arc::clone(&entry.source)))
      .collect();

    for (reference, source) in reloaded {
      source.forget_textures(names);
      self.load(&reference, &source);
    }
  }

  /// Reads and decodes a texture on a loader thread.
  fn load(&self, reference: &str, source: &Arc<dyn RenderAssetSource>) {
    let (sender, source, reference) = (self.sender.clone(), Arc::clone(source), reference.to_owned());

    self.workers.spawn(move || {
      let load: Result<Option<DecodedTexture>, String> = source
        .read_texture(&reference)
        .and_then(|bytes| bytes.map(|bytes| DecodedTexture::from_dds(&bytes)).transpose())
        .map_err(|error| error.to_string());

      if let Err(error) = &load {
        log::warn!("Weather texture '{reference}' cannot be drawn: {error}");
      }

      let _ = sender.send((reference, load));
    });
  }

  /// Asks for every texture a sky draws with; answers whether each is up or given up on, the clouds only where shown.
  pub fn request_sky(&mut self, sky: &RenderSky, is_clouded: bool, source: &Arc<dyn RenderAssetSource>) -> bool {
    let cubes = sky.textures.iter().chain(&sky.environments).flatten();
    let clouds = sky.clouds.textures.iter().flatten().filter(|_| is_clouded);
    let mut is_settled: bool = true;

    for reference in cubes {
      is_settled &= self.request(reference, WeatherTextureKind::Cube, source);
    }

    for reference in clouds {
      is_settled &= self.request(reference, WeatherTextureKind::Flat, source);
    }

    is_settled
  }

  /// Whether a texture is up or given up on for good; none counts as settled.
  pub fn is_settled(&self, reference: Option<&str>) -> bool {
    reference.is_none_or(|reference| self.entries.get(reference).is_some_and(|entry| entry.is_settled))
  }

  /// Whether a texture is up, rather than standing in for itself.
  pub fn is_up(&self, reference: &str) -> bool {
    self.entries.get(reference).is_some_and(|entry| entry.view.is_some())
  }

  /// What samples a texture: its own once up, its kind's placeholder before, or for none.
  pub fn get_view(&self, reference: Option<&str>, kind: WeatherTextureKind) -> &wgpu::TextureView {
    reference
      .and_then(|reference| self.entries.get(reference))
      .and_then(|entry| entry.view.as_ref())
      .unwrap_or(match kind {
        WeatherTextureKind::Cube => &self.placeholder_cube,
        WeatherTextureKind::Flat => &self.placeholder_flat,
        WeatherTextureKind::Volume => &self.placeholder_volume,
      })
  }

  /// Takes what the loaders finished and uploads it, and lets go of what nobody asked for in a while.
  pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    self.frame += 1;

    while let Ok((reference, load)) = self.receiver.try_recv() {
      let Some(entry) = self.entries.get_mut(&reference) else {
        continue;
      };

      entry.is_settled = true;

      match load {
        Ok(Some(texture)) if !texture.is_within(&device.limits()) => {
          log::warn!("Weather texture '{reference}' is larger than the device holds");
        }
        Ok(Some(texture)) if Self::is_kind(&texture, entry.kind) => {
          entry.view = Some(texture.upload(device, queue, &reference));
        }
        Ok(Some(_)) => log::warn!("Weather texture '{reference}' is not the kind it is drawn as"),
        _ => {}
      }
    }

    let frame: u64 = self.frame;

    self
      .entries
      .retain(|_, entry| frame.saturating_sub(entry.used) <= KEPT_FRAMES);
  }

  /// Whether a texture read is the kind it is drawn as.
  fn is_kind(texture: &DecodedTexture, kind: WeatherTextureKind) -> bool {
    match kind {
      WeatherTextureKind::Cube => !texture.is_volume && texture.layers == DecodedTexture::CUBE_FACES,
      WeatherTextureKind::Flat => texture.layers == 1,
      WeatherTextureKind::Volume => texture.is_volume,
    }
  }

  /// One texel of a colour on every layer, viewed as asked.
  fn create_solid(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    color: [u8; 4],
    layers: u32,
    dimension: wgpu::TextureViewDimension,
  ) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;

    let texels: Vec<u8> = (0..layers).flat_map(|_| color).collect();

    device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("sky placeholder"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: layers,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &texels,
      )
      .create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(dimension),
        ..Default::default()
      })
  }
}
