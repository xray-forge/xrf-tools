//! What the level plugin says to the log, in one place rather than beside each command that says it.

use std::time::Instant;

use xrf_level::LevelShaderEntry;
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_visual::{SectorDescription, SectorInstanceGroup, SectorOutline, SectorPackage};

use crate::plugins::levels::state::{LevelSource, LevelTextureReference, SelectedLevel};

/// How many names a log line about a set of them carries before it stops listing and starts counting.
const LISTED_NAMES: usize = 6;

/// Bytes past which a packed sector is worth saying something about.
const LARGE_SECTOR_BYTES: usize = 128 * 1024 * 1024;

/// What one open came to, and what about it is worth a warning.
pub fn report_open(selected: &SelectedLevel, started: Instant) {
  let source: &LevelSource = &selected.source;

  log::info!(
    "Opened level {} in {}: {} sectors, {} visuals, {} drawable, {} textures, {} surfaces",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed()),
    selected.outlines.len(),
    selected.visuals.visuals.len(),
    selected.visuals.count_drawable(),
    selected.textures.len(),
    selected.surfaces.len()
  );

  report_sectors(source, &selected.outlines);
  report_unresolved(source, &selected.textures);
  report_surfaces(source, &named_surfaces(selected));
}

/// The surfaces the table actually dresses something with, each beside the shader it names.
fn named_surfaces(selected: &SelectedLevel) -> Vec<(&str, &XraySurfaceDescriptor)> {
  let Some(shaders) = selected.level.shaders.as_ref() else {
    return Vec::new();
  };

  shaders
    .entries
    .iter()
    .zip(&selected.surfaces)
    .filter_map(|(entry, surface)| match entry {
      LevelShaderEntry::Reference(reference) => Some((reference.shader.as_str(), surface)),
      LevelShaderEntry::Empty | LevelShaderEntry::Malformed(_) => None,
    })
    .collect()
}

/// Says how a level divides, because a level whose sectors are few and huge is one a viewer cannot stream.
fn report_sectors(source: &LevelSource, outlines: &[SectorOutline]) {
  if outlines.is_empty() {
    return;
  }

  let reached: usize = outlines.iter().map(|outline| outline.drawables as usize).sum();

  log::info!(
    "Sectors of {} reach {} drawables, {} each on average, largest {}",
    source.get_label(),
    reached,
    reached / outlines.len(),
    outlines.iter().map(|outline| outline.drawables).max().unwrap_or(0)
  );
}

/// Says which references the roots answer nothing for, and names a few.
fn report_unresolved(source: &LevelSource, textures: &[LevelTextureReference]) {
  let unresolved: Vec<&str> = textures
    .iter()
    .filter(|it| it.logical_path.is_none())
    .map(|it| it.reference.as_str())
    .collect();

  if unresolved.is_empty() {
    return;
  }

  log::warn!(
    "Level {} names {} of {} textures the searched roots hold nothing for: {}",
    source.get_label(),
    unresolved.len(),
    textures.len(),
    name_a_few(&unresolved)
  );
}

/// Says how many surfaces read alpha, how many are detailed, and how many the library could say nothing about.
fn report_surfaces(source: &LevelSource, surfaces: &[(&str, &XraySurfaceDescriptor)]) {
  let alpha: usize = surfaces
    .iter()
    .filter(|(_, surface)| surface.draw != XraySurfaceDraw::Opaque)
    .count();
  let detailed: usize = surfaces.iter().filter(|(_, surface)| surface.detail.is_some()).count();
  let undescribed: Vec<&str> = surfaces
    .iter()
    .filter(|(_, surface)| !matches!(surface.declaration, XraySurfaceDeclaration::Described { .. }))
    .map(|(shader, _)| *shader)
    .collect();

  log::info!(
    "Level {} draws {} of {} surfaces with alpha and {} with a detail texture",
    source.get_label(),
    alpha,
    surfaces.len(),
    detailed
  );

  if !undescribed.is_empty() {
    log::warn!(
      "Level {} names {} shaders the library could not describe, so they draw opaque: {}",
      source.get_label(),
      undescribed.len(),
      name_a_few(&undescribed)
    );
  }
}

/// Says what one sector came to, and says it louder when it came to too much.
pub fn report_packed_sector(package: &SectorPackage, started: Instant) {
  let description: &SectorDescription = &package.description;
  let instances: u32 = description
    .instances
    .iter()
    .map(|group: &SectorInstanceGroup| group.instance_count)
    .sum();

  log::info!(
    "Packed sector {} in {}: {} vertices, {} indices, {} draws, {} instanced meshes standing {} times, {}",
    description.sector,
    xrf_utils::format_duration(started.elapsed()),
    description.geometry.vertex_count,
    description.geometry.index_count,
    description.sections.len(),
    description.instances.len(),
    instances,
    xrf_utils::format_bytes(package.buffer.len() as u64)
  );

  if !description.skipped.is_empty() {
    log::warn!(
      "Sector {} left out {} drawables, first: {}",
      description.sector,
      description.skipped.len(),
      description.skipped[0].reason
    );
  }

  if package.buffer.len() >= LARGE_SECTOR_BYTES {
    log::warn!(
      "Sector {} packed to {}, which is past what a viewer should hold several of: this level's sectors are not a streaming unit",
      description.sector,
      xrf_utils::format_bytes(package.buffer.len() as u64)
    );
  }
}

/// Names the first few of a set and says how many more there are, for a log line that has to stay one line.
fn name_a_few(names: &[&str]) -> String {
  let listed: String = names
    .iter()
    .take(LISTED_NAMES)
    .copied()
    .collect::<Vec<&str>>()
    .join(", ");

  match names.len().saturating_sub(LISTED_NAMES) {
    0 => listed,
    rest => format!("{listed} and {rest} more"),
  }
}
