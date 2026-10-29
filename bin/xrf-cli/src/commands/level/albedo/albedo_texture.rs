use std::collections::BTreeSet;

use serde::Serialize;
use xrf_texture::TextureAlbedo;

/// One base texture the level's albedo-writing surfaces dress with, and how its colours sit as albedo.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbedoTexture {
  /// The texture as the shader table names it.
  pub reference: String,
  /// The file it resolved to, beside the level or in the shared tree.
  pub file: Option<String>,
  /// The shaders drawing it.
  pub shaders: BTreeSet<String>,
  /// The alpha test its surfaces cut it out by, `None` for an opaque one.
  pub alpha_reference: Option<u8>,
  /// Whether a terrain class lays detail textures over it, which makes it a colour map rather than one material.
  pub is_terrain: bool,
  #[serde(flatten)]
  pub outcome: AlbedoOutcome,
}

/// What reading one texture's albedo came to.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AlbedoOutcome {
  Measured {
    albedo: TextureAlbedo,
  },
  /// No root holds it: the renderer binds its stand-in.
  Missing,
  Unread {
    reason: String,
  },
  /// The alpha test kills every texel.
  Empty,
}

impl AlbedoTexture {
  /// Its measure, where it was measured.
  pub fn get_albedo(&self) -> Option<&TextureAlbedo> {
    match &self.outcome {
      AlbedoOutcome::Measured { albedo } => Some(albedo),
      _ => None,
    }
  }
}
