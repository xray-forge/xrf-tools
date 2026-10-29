use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;
use xrf_dds::DdsFile;
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelShaderEntry, LevelShaderReference};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceDraw, XraySurfaceResolver, XrayTextureScope};
use xrf_texture::TextureAlbedo;
use xrf_vfs::{XrayAsset, XraySearch};

use crate::commands::level::albedo::albedo_texture::{AlbedoOutcome, AlbedoTexture};

/// The base textures of a level's surfaces that write albedo, each measured as the renderer reads it.
pub struct LevelAlbedoSurvey {
  /// Shader table entries naming a surface.
  pub surfaces: usize,
  /// Of those, the ones writing no albedo: blended, added, multiplied, water, invisible or self-lit.
  pub unlit_surfaces: usize,
  /// One a texture and alpha test, in name order.
  pub textures: Vec<AlbedoTexture>,
}

impl LevelAlbedoSurvey {
  /// Surveys the shader table of `level`, its textures found within `scope` as the renderer finds them.
  pub fn run(search: &XraySearch, scope: XrayTextureScope, level: &LevelFile) -> Self {
    let resolver: XraySurfaceResolver = XraySurfaceResolver::open(search, scope.clone());
    let references: Vec<&LevelShaderReference> = level
      .shaders
      .iter()
      .flat_map(|shaders| &shaders.entries)
      .filter_map(|entry| match entry {
        LevelShaderEntry::Reference(reference) => Some(reference),
        _ => None,
      })
      .collect();
    let mut gathered: BTreeMap<(String, Option<u8>), AlbedoTexture> = BTreeMap::new();
    let mut unlit_surfaces: usize = 0;

    for reference in &references {
      let surface: XraySurfaceDescriptor = resolver.describe(&reference.shader, &reference.textures);
      let alpha_reference: Option<u8> = match surface.draw {
        XraySurfaceDraw::Opaque => None,
        XraySurfaceDraw::AlphaTested { reference } => Some(reference),
        _ => {
          unlit_surfaces += 1;
          continue;
        }
      };

      let Some(base) = reference.textures.first().filter(|_| !surface.is_emissive) else {
        unlit_surfaces += 1;
        continue;
      };

      let texture: &mut AlbedoTexture = gathered
        .entry((base.to_lowercase(), alpha_reference))
        .or_insert_with(|| AlbedoTexture {
          reference: base.clone(),
          file: None,
          shaders: BTreeSet::new(),
          alpha_reference,
          is_terrain: false,
          // Until the measure below finds its file.
          outcome: AlbedoOutcome::Missing,
        });

      texture.shaders.insert(reference.shader.clone());
      texture.is_terrain |= surface.terrain.is_some();
    }

    let mut textures: Vec<AlbedoTexture> = gathered.into_values().collect();

    // Decoding is the survey's whole cost, and each texture's is its own.
    textures
      .par_iter_mut()
      .for_each(|texture| Self::measure(search, &scope, texture));

    Self {
      surfaces: references.len(),
      unlit_surfaces,
      textures,
    }
  }

  /// Finds and measures one texture, recording why it could not be.
  fn measure(search: &XraySearch, scope: &XrayTextureScope, texture: &mut AlbedoTexture) {
    let asset: Option<XrayAsset> = scope
      .resolve_texture(search, &texture.reference)
      .ok()
      .and_then(|resolution| resolution.get_asset().cloned());
    let Some(asset) = asset else {
      return;
    };

    texture.file = Some(asset.get_logical_path().as_str().to_owned());
    texture.outcome = match Self::read(search, &asset, texture.alpha_reference) {
      Ok(Some(albedo)) => AlbedoOutcome::Measured { albedo },
      Ok(None) => AlbedoOutcome::Empty,
      Err(error) => AlbedoOutcome::Unread {
        reason: error.to_string(),
      },
    };
  }

  fn read(search: &XraySearch, asset: &XrayAsset, alpha_reference: Option<u8>) -> XrfResult<Option<TextureAlbedo>> {
    TextureAlbedo::read(&DdsFile::read_from_bytes(&search.read_asset(asset)?)?, alpha_reference)
  }
}
