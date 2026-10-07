use std::sync::Arc;

use xrf_error::XrfResult;

use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_details::RenderLevelDetails;
use crate::host::render_level_source::RenderLevelSource;
use crate::pass::grass_pass::GrassPass;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::loader_answer::take_answer;
use crate::scene::texture::texture_cache::TextureCache;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// A level's grass (`CDetailManager`): its slots and models, read once on a loader thread; each view plants and draws
/// it around its own camera (`GrassView`).
pub struct LevelGrass {
  pending: Option<LoaderReceiver<XrfResult<Option<RenderLevelDetails>>>>,
  level: Option<GrassLevel>,
}

impl LevelGrass {
  pub fn new(source: &Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();
    let source: Arc<dyn RenderLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let read: XrfResult<Option<RenderLevelDetails>> = source.read_details();

      if let Err(error) = &read {
        log::warn!("The level's grass cannot be drawn: {error}");
      }

      let _ = sender.send(read);
    });

    Self {
      pending: Some(receiver),
      level: None,
    }
  }

  /// Whether its loader has answered, whatever it answered.
  pub fn is_loaded(&self) -> bool {
    self.pending.is_none()
  }

  /// What was read, none before it is or where the level has no grass.
  pub fn get_level(&self) -> Option<&GrassLevel> {
    self.level.as_ref()
  }

  /// Takes the level's grass once it is read, asking for its textures; answers their slots the first time.
  pub fn poll(
    &mut self,
    device: &wgpu::Device,
    pass: &GrassPass,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> Option<Vec<u32>> {
    let read: XrfResult<Option<RenderLevelDetails>> = take_answer(&mut self.pending, "grass")?;
    let details: RenderLevelDetails = read.ok().flatten()?;
    let level: GrassLevel = GrassLevel::new(device, pass, &details, (textures, source));
    let slots: Vec<u32> = level.texture_slots.clone();

    log::info!(
      "Native viewport grass {} models over {}x{} slots",
      level.model_count,
      level.grid[0],
      level.grid[1]
    );
    self.level = Some(level);

    Some(slots)
  }
}
