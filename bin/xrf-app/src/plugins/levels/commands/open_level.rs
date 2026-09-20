use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::State;
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::read::{ReadLevel, read_source};
use crate::plugins::levels::state::{
  LevelSource, LevelState, LevelTextureReference, SelectedLevel, SelectedLevelDescription,
};
use crate::plugins::levels::surfaces::resolve_surfaces;
use crate::plugins::levels::textures::resolve_textures;

/// How many names a log line about a set of them carries before it stops listing and starts counting.
const LISTED_NAMES: usize = 6;

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_level"))]
#[tauri::command(rename = "open_level")]
pub async fn levels_open_level(
  session_id: SessionId,
  source: LevelSource,
  roots: XrayRoots,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SessionSnapshot<SelectedLevelDescription>> {
  state.selected.begin_open(session_id)?;

  let started: Instant = Instant::now();

  log::info!("Opening level: {}", source.get_label());

  let roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let directory: Option<String> = source.get_logical_directory();
  let (read, textures, surfaces) = assets.with_probe(&roots, |probe| {
    let read: ReadLevel = read_source(&source, probe)?;
    let textures: Vec<LevelTextureReference> = resolve_textures(&read.level, probe, directory.as_deref());
    let surfaces: HashMap<String, XraySurfaceDescriptor> = resolve_surfaces(&read.level, probe);

    TauriResult::Ok((read, textures, surfaces))
  })??;

  let outlines: Vec<SectorOutline> = read
    .level
    .sectors
    .as_ref()
    .map_or(&[][..], |chunk| &chunk.sectors)
    .iter()
    .enumerate()
    .map(|(index, sector)| SectorOutline::of(&read.visuals, index as u32, sector.root))
    .collect();

  let drawables: usize = read.visuals.count_drawable();
  let reached: usize = outlines.iter().map(|outline| outline.drawables as usize).sum();

  log::info!(
    "Opened level {} in {}: {} sectors, {} visuals, {} drawable, {} textures, {} surfaces",
    source.get_label(),
    xrf_utils::format_duration(started.elapsed()),
    outlines.len(),
    read.visuals.visuals.len(),
    drawables,
    textures.len(),
    surfaces.len()
  );

  // A level whose sectors are few and huge is one a viewer cannot stream, and it is worth knowing before the first
  // sector is packed rather than when the pack runs out of memory.
  if !outlines.is_empty() {
    log::info!(
      "Sectors of {} reach {} drawables, {} each on average, largest {}",
      source.get_label(),
      reached,
      reached / outlines.len(),
      outlines.iter().map(|outline| outline.drawables).max().unwrap_or(0)
    );
  }

  report_unresolved(&source, &textures);
  report_surfaces(&source, &surfaces);

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.commit_open(
    session_id,
    SelectedLevel {
      geometry: Mutex::new(read.geometry),
      level: read.level,
      outlines,
      packed: Session::new("level sector"),
      roots,
      source,
      surfaces,
      textures,
      visuals: read.visuals,
    },
  )?;

  Ok(selected.map(SelectedLevel::describe))
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

/// Says how many surfaces read alpha, and how many the shader library could say nothing about.
fn report_surfaces(source: &LevelSource, surfaces: &HashMap<String, XraySurfaceDescriptor>) {
  let alpha: usize = surfaces
    .values()
    .filter(|it| it.draw != XraySurfaceDraw::Opaque)
    .count();
  let undescribed: Vec<&str> = surfaces
    .iter()
    .filter(|(_, descriptor)| !matches!(descriptor.declaration, XraySurfaceDeclaration::Described { .. }))
    .map(|(shader, _)| shader.as_str())
    .collect();

  log::info!(
    "Level {} draws {} of {} surfaces with alpha",
    source.get_label(),
    alpha,
    surfaces.len()
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
