use xrf_renderer_core::{FrameGraph, GraphBindings, GraphTexture};

use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::weather::sky::deband::sky_deband_parameters::{SkyDebandCubeParameters, SkyDebandFlatParameters};
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::lighting::weather::render_sky::RenderSky;
use crate::pass::level_passes::LevelPasses;
use crate::scene::texture::weather_textures::WeatherTextures;

/// The sky's textures as the frame draws them, smoothed where their stored steps band (`common/sky_deband.wgsl`): both
/// keyframes' cubes and clouds, each copied into float once its texture is up and again when the weather names
/// another, the combine reading the copies and every other reader the textures as they are.
#[derive(Default)]
pub struct SkyDeband {
  /// The cubes' copies, then the clouds', none for a texture not up or not drawn.
  slots: [Option<SkyDebandSlot>; 4],
}

/// One texture's smoothed copy: the texture it was made from, at what size, the copy as the sky reads it and as the
/// smoothing writes it, and whether this frame smooths it.
struct SkyDebandSlot {
  reference: String,
  size: (u32, u32),
  sampled: wgpu::TextureView,
  written: wgpu::TextureView,
  is_due: bool,
  is_filled: bool,
}

impl SkyDeband {
  const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Each slot's pass, the cubes' first.
  const PASSES: [&'static str; 4] = [
    "sky deband cube 0",
    "sky deband cube 1",
    "sky deband clouds 0",
    "sky deband clouds 1",
  ];

  /// Whether a view debands its sky: the enhanced debanding asked for, the sky shown, lit and solid.
  pub fn is_asked(options: &RenderViewOptions) -> bool {
    let is_shown: bool = options.mode.is_lit && !options.mode.is_wireframe && options.show.is_sky_visible;

    is_shown && options.features.debanding.is_drawn()
  }

  /// Takes this frame's sky: a texture up for the first time, or named in place of the one copied, is copied again this
  /// frame; one not up, or clouds not drawn (`is_clouded`), keeps no copy, the sky reading it as it stands.
  pub fn prepare(&mut self, device: &wgpu::Device, textures: WeatherTextures<'_>, sky: &RenderSky, is_clouded: bool) {
    let references: [Option<&str>; 4] = [
      sky.textures[0].as_deref(),
      sky.textures[1].as_deref(),
      sky.clouds.textures[0].as_deref().filter(|_| is_clouded),
      sky.clouds.textures[1].as_deref().filter(|_| is_clouded),
    ];

    for (index, (slot, reference)) in self.slots.iter_mut().zip(references).enumerate() {
      let is_cube: bool = index < 2;
      let up: Option<(&str, (u32, u32))> =
        reference.and_then(|reference| textures.get_size(reference).map(|size| (reference, size)));

      match (slot.as_mut(), up) {
        (_, None) => *slot = None,
        (Some(kept), Some((reference, size))) if kept.reference == reference && kept.size == size => {
          kept.is_due = !kept.is_filled;
          kept.is_filled = true;
        }
        (_, Some((reference, size))) => {
          *slot = Some(SkyDebandSlot::new(device, reference, size, is_cube));
        }
      }
    }
  }

  /// Forgets a smoothing this frame made, where its frame was not submitted: it smooths again on the next.
  pub fn abandon_frame(&mut self) {
    for slot in self.slots.iter_mut().flatten().filter(|slot| slot.is_due) {
      slot.is_filled = false;
    }
  }

  /// The sky's parameters (`sky`) with the copies in place of the textures they were made from, each smoothed first
  /// where this frame copies it.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    passes: LevelPasses<'a>,
    sky: SkyParameters<'a>,
  ) -> SkyParameters<'a> {
    let sources: [GraphTexture; 4] = [sky.sky_cube_0, sky.sky_cube_1, sky.sky_clouds_0, sky.sky_clouds_1];
    let mut drawn: [GraphTexture; 4] = sources;

    for (index, slot) in self.slots.iter().enumerate() {
      let Some(slot) = slot else {
        continue;
      };

      drawn[index] = bindings.import_view(&mut *graph, "sky debanded", &slot.sampled);

      if !slot.is_due {
        continue;
      }

      let written: GraphTexture = bindings.import_view(&mut *graph, "sky debanding", &slot.written);
      let pass = &passes.weather.sky_deband;
      let (width, height) = slot.size;

      if index < 2 {
        let parameters: SkyDebandCubeParameters<'_> = SkyDebandCubeParameters {
          cube: sources[index],
          cube_sampler: sky.sky_clamp,
          faces: written,
        };

        graph
          .add_compute_pass(Self::PASSES[index])
          .parameters(&parameters)
          .record(move |context| pass.record_cube(context, width, &parameters));
      } else {
        let parameters: SkyDebandFlatParameters<'_> = SkyDebandFlatParameters {
          clouds: sources[index],
          clouds_sampler: sky.sky_repeat,
          smoothed: written,
        };

        graph
          .add_compute_pass(Self::PASSES[index])
          .parameters(&parameters)
          .record(move |context| pass.record_flat(context, (width, height), &parameters));
      }
    }

    SkyParameters {
      sky_cube_0: drawn[0],
      sky_cube_1: drawn[1],
      sky_clouds_0: drawn[2],
      sky_clouds_1: drawn[3],
      ..sky
    }
  }

  /// The copies' memory: eight bytes a texel, a cube's six faces.
  pub fn get_memory(&self) -> u64 {
    self
      .slots
      .iter()
      .enumerate()
      .filter_map(|(index, slot)| {
        let faces: u64 = if index < 2 { 6 } else { 1 };

        slot
          .as_ref()
          .map(|slot| u64::from(slot.size.0) * u64::from(slot.size.1) * 8 * faces)
      })
      .sum()
  }
}

impl SkyDebandSlot {
  /// A float copy of a texture `size` texels a side (a face's, six faces for a cube), smoothed on the frame it is made.
  fn new(device: &wgpu::Device, reference: &str, size: (u32, u32), is_cube: bool) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("sky debanded"),
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: if is_cube { 6 } else { 1 },
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: SkyDeband::FORMAT,
      usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
    let view = |dimension: wgpu::TextureViewDimension| {
      texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("sky debanded"),
        dimension: Some(dimension),
        ..Default::default()
      })
    };
    let (sampled, written): (wgpu::TextureView, wgpu::TextureView) = if is_cube {
      (
        view(wgpu::TextureViewDimension::Cube),
        view(wgpu::TextureViewDimension::D2Array),
      )
    } else {
      (
        view(wgpu::TextureViewDimension::D2),
        view(wgpu::TextureViewDimension::D2),
      )
    };

    Self {
      reference: reference.to_owned(),
      size,
      sampled,
      written,
      is_due: true,
      is_filled: true,
    }
  }
}
