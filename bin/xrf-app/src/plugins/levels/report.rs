//! What the level plugin says to the log, in one place rather than beside each command that says it.

use std::fmt::Display;
use std::time::{Duration, Instant};

use xrf_level::{LevelSectorComposition, LevelShaderEntry};
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_spawn::SpawnLevelObjects;
use xrf_visual::{
  DetailsDescription, LightsDescription, SectorDescription, SectorInstanceGroup, SectorOutline, SectorPackage,
};

use crate::plugins::levels::state::{
  LevelEntry, LevelSource, LevelSpawnObjectsDescription, LevelStart, LevelTextureReference, SelectedLevel,
};

/// How many names a log line about a set of them carries before it stops listing and starts counting.
const LISTED_NAMES: usize = 6;

/// Bytes past which a packed sector is worth saying something about.
const LARGE_SECTOR_BYTES: usize = 256 * 1024 * 1024;

/// How many levels the roots hold, and how many of them draw.
pub fn report_listed_levels(entries: &[LevelEntry], started: Instant) {
  log::info!(
    "Listed {} levels in {}, {} of them with render geometry",
    entries.len(),
    xrf_utils::format_duration(started.elapsed()),
    entries.iter().filter(|entry| entry.has_geometry).count()
  );
}

/// That an open has begun, before anything is read.
pub fn report_opening(source: &LevelSource) {
  log::info!("Opening level: {}", source.get_label());
}

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

/// What one sector is about to be packed out of.
pub fn report_packing_sector(sector: u32, root: u32, composition: &LevelSectorComposition) {
  log::info!(
    "Packing sector {sector} of root {root}: {} drawables, {} hierarchies",
    composition.drawables.len(),
    composition.hierarchies.len()
  );
}

/// Where a level opens, and what that was taken from.
pub fn report_start(source: &LevelSource, start: Option<&LevelStart>) {
  match start {
    Some(start) => log::info!(
      "Level {} opens at {:?}, from its {:?}",
      source.get_label(),
      start.position,
      start.origin
    ),
    None => log::info!(
      "Level {} names nowhere to open, so a viewer frames it",
      source.get_label()
    ),
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

/// What a level's share of the spawn came to, and how much of the spawn could not be read at all.
pub fn report_spawn(level: &str, file: &str, read: &SpawnLevelObjects, started: Instant) {
  log::info!(
    "Read the spawn of {level} in {}: {} of {} objects, {} arrivals",
    xrf_utils::format_duration(started.elapsed()),
    read.objects.len(),
    read.total,
    read.arrivals.len()
  );

  if let Some(first) = read.skipped.first() {
    log::warn!(
      "Skipped {} of the {} objects of '{file}', as they could not be read; the first, object {}: {}",
      read.skipped.len(),
      read.total,
      first.index,
      first.reason
    );
  }
}

/// How many of a level's spawned objects are left out as a new game releases them.
pub fn report_new_game_releases(level: &str, released: usize) {
  if released > 0 {
    log::info!("Marked {released} spawned objects of {level} that a new game releases");
  }
}

/// That a list of the objects a new game releases could not be read, so they are drawn.
pub fn report_unreadable_releases(file: &str, error: &impl Display) {
  log::warn!("Objects '{file}' releases on a new game are kept, as it is unreadable: {error}");
}

/// How many of the resolved configs' sections a level keeps, which is the ones its spawned objects name.
pub fn report_sections(source: &LevelSource, kept: usize, resolved: usize, started: Instant) {
  log::info!(
    "Read the configs of {} in {}: kept {kept} of {resolved} sections",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed())
  );
}

/// That the configs could not be read, so zones light nothing and lamps keep their spawned flags.
pub fn report_missing_sections(error: &impl Display) {
  log::warn!("No zone lights and no lamp sections: {error}");
}

/// That the light animation library could not be read, so no light is animated.
pub fn report_unreadable_animations(file: &str, error: &impl Display) {
  log::warn!("No light is animated, as '{file}' is unreadable: {error}");
}

/// That the spawn could not be read, so the level keeps its own lights alone.
pub fn report_missing_spawned_lights(source: &LevelSource, error: &impl Display) {
  log::warn!("No spawned lights for {}: {error}", source.get_label());
}

/// What a level's lights came to.
pub fn report_lights(source: &LevelSource, lights: &LightsDescription, started: Instant) {
  log::info!(
    "Collected lights of {} in {}: {} lights, {} animators, {} projectors",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed()),
    lights.lights.len(),
    lights.animators.len(),
    lights.projectors.len()
  );
}

/// That the spawn could not be read, so the level stands no spawned object.
pub fn report_missing_spawn_objects(source: &LevelSource, error: &impl Display) {
  log::warn!("No spawned objects for {}: {error}", source.get_label());
}

/// That a spawned visual could not be read, so it is not drawn.
pub fn report_undrawn_visual(name: &str, error: &impl Display) {
  log::warn!("Spawned visual '{name}' is not drawn: {error}");
}

/// That a spawned visual's rest cycle holds no frame, so it stands in its bind pose.
pub fn report_empty_rest_motion(name: &str, motion: &str) {
  log::debug!("Spawned visual '{name}' has an empty '{motion}' cycle, so it stands in its bind pose");
}

/// That a spawned visual's rest cycle could not be baked, so it stands in its bind pose.
pub fn report_bind_rest_pose(name: &str, error: &impl Display) {
  log::debug!("Spawned visual '{name}' stands in its bind pose: {error}");
}

/// What a level's spawned objects came to: how many are drawn, and how many visuals they stand as.
pub fn report_spawn_objects(source: &LevelSource, objects: &LevelSpawnObjectsDescription, started: Instant) {
  log::info!(
    "Described the spawned objects of {} in {}: {} objects, {} visuals",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed()),
    objects.objects.len(),
    objects.visuals.len()
  );
}

/// What the estimate of how a level lights its spawned objects was built from, and how long its form took to read.
pub fn report_hemi(source: &LevelSource, triangles: usize, lights: usize, read: Duration, started: Instant) {
  log::info!(
    "Built the spawned objects' lighting of {} in {} ({} reading the form): {triangles} collision triangles, {lights} \
     compiled point lights",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed()),
    xrf_utils::format_duration(read)
  );
}

/// That the collision form could not be read, so every spawned object is lit as if under the open sky.
pub fn report_missing_hemi(source: &LevelSource, error: &impl Display) {
  log::warn!(
    "Spawned objects of {} are lit as under the open sky: {error}",
    source.get_label()
  );
}

/// That the compiled lights could not be read, so they light no spawned object.
pub fn report_unreadable_lights(source: &LevelSource, error: &impl Display) {
  log::warn!(
    "Compiled lights of {} light no spawned object: {error}",
    source.get_label()
  );
}

/// That the game material library could not be read, so every collision triangle is solid ground.
pub fn report_unreadable_materials(file: &str, error: &impl Display) {
  log::warn!("Every collision triangle is taken as solid ground, as '{file}' is unreadable: {error}");
}

/// What a level's grass came to, or that it has none.
pub fn report_details(source: &LevelSource, details: Option<&DetailsDescription>, started: Instant) {
  let Some(details) = details else {
    log::info!("Level {} has no detail library", source.get_label());

    return;
  };

  log::info!(
    "Packed grass of {} in {}: {} planted slots, {} triangles, {} bin entries, {} models, {}",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed()),
    details.slot_count,
    details.get_triangle_count(),
    details.get_bin_length(),
    details.models.len(),
    xrf_utils::format_bytes(u64::from(details.buffer_length))
  );
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
