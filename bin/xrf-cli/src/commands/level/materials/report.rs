use std::collections::BTreeSet;

use serde::Serialize;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelShaderEntry, LevelShadersChunk, LevelSurfaceMaterial, LevelSurfaceMaterials};

/// What `level materials` found: the level at a glance, then each shader table entry and what it lies on.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMaterialsReport {
  pub level: String,
  pub summary: LevelMaterialsSummary,
  pub surfaces: Vec<LevelMaterialsSurface>,
}

/// The level's drawn area taken together.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMaterialsSummary {
  /// Shader table entries some drawable visual draws with.
  pub surfaces: usize,
  /// Square metres every drawable visual covers.
  pub area: f64,
  /// The share of it lying on some collision face.
  pub covered_share: f64,
  /// Game materials some surface lies on.
  pub materials: usize,
  /// Visuals whose geometry could not be read.
  pub unread_visuals: usize,
}

/// One shader table entry and the game materials beneath it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMaterialsSurface {
  /// Its index in the shader table, which visuals name it by.
  pub index: u16,
  pub shader: Option<String>,
  pub textures: Vec<String>,
  /// Square metres its triangles cover.
  pub area: f64,
  /// The share of that area lying on no collision face.
  pub uncovered_share: f64,
  /// The materials beneath it, the most covered first, each with its share of the covered area.
  pub materials: Vec<LevelMaterialsShare>,
}

/// A game material's share of a surface's covered area.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMaterialsShare {
  /// Its id in `gamemtl.xr`, as the collision form names it.
  pub id: u16,
  /// Its name, where the roots hold a library naming it.
  pub name: Option<String>,
  pub share: f64,
}

impl LevelMaterialsReport {
  /// The report of `survey`, its entries named from the level's shader table and its materials from `library`.
  pub fn new(
    level: String,
    survey: &LevelSurfaceMaterials,
    shaders: Option<&LevelShadersChunk>,
    library: Option<&GameMtlFile>,
  ) -> Self {
    let surfaces: Vec<LevelMaterialsSurface> = survey
      .surfaces
      .iter()
      .map(|(index, tally)| LevelMaterialsSurface::of(*index, tally, shaders, library))
      .collect();
    let area: f64 = survey.surfaces.values().map(|tally| tally.area).sum();
    let uncovered: f64 = survey.surfaces.values().map(|tally| tally.uncovered_area).sum();

    Self {
      level,
      summary: LevelMaterialsSummary {
        surfaces: surfaces.len(),
        area,
        covered_share: if area > 0.0 { 1.0 - uncovered / area } else { 0.0 },
        materials: survey
          .surfaces
          .values()
          .flat_map(|tally| tally.materials.keys())
          .collect::<BTreeSet<_>>()
          .len(),
        unread_visuals: survey.unread_visuals,
      },
      surfaces,
    }
  }
}

impl LevelMaterialsSurface {
  fn of(
    index: u16,
    tally: &LevelSurfaceMaterial,
    shaders: Option<&LevelShadersChunk>,
    library: Option<&GameMtlFile>,
  ) -> Self {
    let (shader, textures): (Option<String>, Vec<String>) =
      match shaders.and_then(|it| it.entries.get(usize::from(index))) {
        Some(LevelShaderEntry::Reference(reference)) => (Some(reference.shader.clone()), reference.textures.clone()),
        _ => (None, Vec::new()),
      };
    let covered: f64 = tally.area - tally.uncovered_area;
    let mut materials: Vec<LevelMaterialsShare> = tally
      .materials
      .iter()
      .map(|(id, area)| LevelMaterialsShare {
        id: *id,
        name: library
          .and_then(|library| library.find_material(u32::from(*id)))
          .map(|material| material.name.clone()),
        share: if covered > 0.0 { area / covered } else { 0.0 },
      })
      .collect();

    materials.sort_by(|first, second| second.share.total_cmp(&first.share));

    Self {
      index,
      shader,
      textures,
      area: tally.area,
      uncovered_share: if tally.area > 0.0 {
        tally.uncovered_area / tally.area
      } else {
        0.0
      },
      materials,
    }
  }
}
