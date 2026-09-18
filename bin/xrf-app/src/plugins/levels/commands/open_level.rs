use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::State;
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::read::{ReadLevel, read_source};
use crate::plugins::levels::state::{
  LevelSource, LevelState, LevelTextureReference, SelectedLevel, SelectedLevelDescription,
};
use crate::plugins::levels::textures::resolve_textures;

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

  log::info!("Opening level: {}", source.label());

  let roots: XrayRoots = roots.centred_on(source.physical_path());
  let (read, textures) = assets.with_probe(&roots, |probe| {
    let read: ReadLevel = read_source(&source, probe)?;
    let textures: Vec<LevelTextureReference> = resolve_textures(&read.level, probe);

    TauriResult::Ok((read, textures))
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
    "Opened level {} in {}: {} sectors, {} visuals, {} drawable, {} textures",
    source.label(),
    xrf_utils::format_duration(started.elapsed()),
    outlines.len(),
    read.visuals.visuals.len(),
    drawables,
    textures.len()
  );

  // A level whose sectors are few and huge is one a viewer cannot stream, and it is worth knowing before the first
  // sector is packed rather than when the pack runs out of memory.
  if !outlines.is_empty() {
    log::info!(
      "Sectors of {} reach {} drawables, {} each on average, largest {}",
      source.label(),
      reached,
      reached / outlines.len(),
      outlines.iter().map(|outline| outline.drawables).max().unwrap_or(0)
    );
  }

  let unresolved: usize = textures.iter().filter(|it| it.logical_path.is_none()).count();

  if unresolved > 0 {
    log::warn!(
      "Level {} names {} textures the mounted roots hold nothing for",
      source.label(),
      unresolved
    );
  }

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.commit_open(
    session_id,
    SelectedLevel {
      geometry: Mutex::new(read.geometry),
      level: read.level,
      outlines,
      packed: Session::new("level sector"),
      roots,
      source,
      textures,
      visuals: read.visuals,
    },
  )?;

  Ok(selected.map(SelectedLevel::describe))
}
