use std::sync::Arc;

use xrf_error::XrfResult;

use crate::contract::report::texture::render_texture_state::RenderTextureState;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::host::render_asset_source::{RenderAssetSource, get_texture_name};
use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::texture_cache::{
  TextureCache, TextureLoad, UPLOAD_BYTES, UPLOAD_PASSES, UPLOAD_SEED, UPLOAD_WAIT_MOST, UPLOADS_OFFERED,
};
use crate::scene::texture::texture_role::TextureRole;
use crate::scene::texture::texture_upload_pace::TextureUploadPace;

impl TextureCache {
  /// Loads again every texture of these file names (lowercase, without folder or extension), whatever its scope, and a
  /// terrain's heights where its bump is among them, their sources first forgetting where they resolved: their files
  /// changed. Each keeps what it shows until its new levels land.
  pub(crate) fn reload(&mut self, names: &[String]) {
    let mut forgotten: Vec<Arc<dyn RenderAssetSource>> = Vec::new();

    for slot in 1..self.views.len() {
      let (Some(_), Some(source)) = (&self.keys[slot], self.sources[slot].clone()) else {
        continue;
      };

      if self.states[slot] == RenderTextureState::Deferred || !self.is_named(slot, names) {
        continue;
      }

      if !forgotten.iter().any(|known| Arc::ptr_eq(known, &source)) {
        source.forget_textures(names);
        forgotten.push(source);
      }

      self.generations[slot] = self.generations[slot].wrapping_add(1);
      self.load(slot as u32);
    }
  }

  /// Whether a slot reads a file of one of these names: its own, or its bump where it is a terrain's heights.
  fn is_named(&self, slot: usize, names: &[String]) -> bool {
    let reference: &str = &self.references[slot];
    let own: String = get_texture_name(reference);
    let bump: Option<String> = (self.roles[slot] == TextureRole::TerrainHeight).then(|| {
      get_texture_name(&format!(
        "{}{}",
        reference.strip_suffix(TextureRole::HEIGHT_SUFFIX).unwrap_or(reference),
        TextureRole::BUMP_SUFFIX
      ))
    });

    names.iter().any(|name| *name == own || bump.as_ref() == Some(name))
  }

  /// Loads a slot again on a loader thread where the stamp of its file no longer matches what it was read at.
  pub(super) fn recheck(&self, slot: u32) {
    let index: usize = slot as usize;
    let Some(source) = self.sources[index].clone() else {
      return;
    };
    let (reference, recorded) = (self.references[index].clone(), self.stamps[index]);

    if recorded.is_none() || self.states[index] == RenderTextureState::Deferred {
      return;
    }

    let (sender, generation, role, lod) = (
      self.sender.clone(),
      self.generations[index],
      self.roles[index],
      self.lod,
    );

    self.workers.spawn(move || {
      if source.get_texture_stamp(&reference) != recorded {
        let _ = sender.send(Self::read_slot(&source, slot, generation, &reference, role, lod));
      }
    });
  }

  /// Reads and decodes a slot's texture on a loader thread, its levels cut as the engine cuts them at `texture_lod`.
  pub(super) fn load(&self, slot: u32) {
    let index: usize = slot as usize;
    let Some(source) = self.sources[index].clone() else {
      return;
    };
    let (sender, generation, reference, role, lod) = (
      self.sender.clone(),
      self.generations[index],
      self.references[index].clone(),
      self.roles[index],
      self.lod,
    );

    self.workers.spawn(move || {
      let _ = sender.send(Self::read_slot(&source, slot, generation, &reference, role, lod));
    });
  }

  /// What reading a slot's texture comes to, with the stamp of the file it was read from.
  fn read_slot(
    source: &Arc<dyn RenderAssetSource>,
    slot: u32,
    generation: u32,
    reference: &str,
    role: TextureRole,
    lod: u32,
  ) -> TextureLoad {
    // Taken before the read, so a write landing during it is read again later rather than missed.
    let stamp: Option<u64> = source.get_texture_stamp(reference);
    // At full quality the listed textures load whole too, so the list is never read.
    let dropped: u32 = Self::get_dropped_levels(lod, lod > 0 && source.is_texture_reduced(reference));
    let read = |reference: &str| -> XrfResult<Option<DecodedTexture>> {
      source
        .read_texture(reference)?
        .map(|bytes| DecodedTexture::from_dds(&bytes).map(|texture| texture.without_levels(dropped)))
        .transpose()
    };
    let load: Result<Option<DecodedTexture>, String> = match role {
      // A detail's height where the installation brings one, else integrated from its bump.
      TextureRole::TerrainHeight => read(reference)
        .and_then(|own| match own {
          Some(own) => Ok(Some(own)),
          None => {
            let bump: String = format!(
              "{}{}",
              reference.strip_suffix(TextureRole::HEIGHT_SUFFIX).unwrap_or(reference),
              TextureRole::BUMP_SUFFIX
            );

            source
              .read_texture(&bump)?
              .map(|bytes| DecodedTexture::from_bump_heights(&bytes, dropped))
              .transpose()
          }
        })
        .map_err(|error| error.to_string()),
      _ => read(reference).map_err(|error| error.to_string()),
    };

    if let Err(error) = &load {
      log::warn!("Texture '{reference}' cannot be drawn: {error}");
    }

    (slot, generation, load, stamp)
  }

  /// Takes what the loaders finished and uploads it at `pace`, then rebinds the array if a slot changed: up to
  /// `UPLOAD_BYTES` a frame while a level opens, and as `scheduler` grants them, in a round of their own, once every
  /// level stands.
  pub(crate) fn update(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scheduler: &mut PeriodicScheduler,
    pace: TextureUploadPace,
  ) {
    while let Ok((slot, generation, load, stamp)) = self.receiver.try_recv() {
      if self.generations.get(slot as usize) != Some(&generation) {
        continue;
      }

      self.stamps[slot as usize] = stamp;

      match load {
        Ok(Some(texture)) => self.uploads.push_back((slot, generation, texture)),
        Ok(None) => self.stand_in(slot, RenderTextureState::Missing),
        Err(reason) => self.stand_in(slot, RenderTextureState::Failed { reason }),
      }
    }

    // Those whose slot was asked for again since are dropped.
    self
      .uploads
      .retain(|(slot, generation, _)| self.generations[*slot as usize] == *generation);

    if pace == TextureUploadPace::Opening {
      let mut spent: u64 = 0;

      // The first upload of a frame always goes, so one texture larger than the budget is not held forever.
      while spent < UPLOAD_BYTES
        && let Some((slot, _, texture)) = self.uploads.pop_front()
      {
        spent += texture.get_size();
        self.upload(device, queue, slot, &texture);
      }

      self.waited = 0;
    } else if !self.uploads.is_empty() {
      let count: usize = self.uploads.len().min(UPLOADS_OFFERED);
      let tickets: Vec<PeriodicTicket> = self
        .uploads
        .iter()
        .take(count)
        .map(|(_, _, texture)| {
          scheduler.offer(PeriodicJob::new_waiting(
            &UPLOAD_PASSES,
            UPLOAD_SEED,
            texture.get_size() as f32,
            self.waited,
            UPLOAD_WAIT_MOST,
          ))
        })
        .collect();

      scheduler.grant();

      let offered: Vec<(u32, u32, DecodedTexture)> = self.uploads.drain(..count).collect();
      let mut kept: Vec<(u32, u32, DecodedTexture)> = Vec::new();

      for ((slot, generation, texture), ticket) in offered.into_iter().zip(tickets) {
        if scheduler.is_granted(ticket) {
          self.upload(device, queue, slot, &texture);
        } else {
          kept.push((slot, generation, texture));
        }
      }

      self.waited = if kept.is_empty() { 0 } else { self.waited + 1 };

      for item in kept.into_iter().rev() {
        self.uploads.push_front(item);
      }
    }

    if self.is_dirty {
      self.is_dirty = false;
      self.bind_group = Self::create_bind_group(device, &self.layout, &self.sampler, &self.views);
    }
  }

  /// Uploads a loaded texture into its slot.
  fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, slot: u32, texture: &DecodedTexture) {
    // One view the device refuses would make the whole bindless group invalid, and every surface with it.
    let limits: wgpu::Limits = device.limits();

    if !texture.is_within(&limits) {
      let reason: String = format!(
        "{} by {} texels, past the {} a side the device holds",
        texture.width, texture.height, limits.max_texture_dimension_2d
      );

      self.stand_in(slot, RenderTextureState::Failed { reason });

      return;
    }

    // The bindless array holds two dimensional textures alone, so a cube or a volume named as a level texture is
    // refused rather than drawn by its first face.
    if !texture.is_flat() {
      let kind: &str = if texture.is_volume { "a volume" } else { "a cube map" };

      self.stand_in(
        slot,
        RenderTextureState::Failed {
          reason: format!("It is {kind}, and a level texture is drawn flat"),
        },
      );

      return;
    }

    self.views[slot as usize] = texture.upload(device, queue, "level texture");
    self.blue_free[slot as usize] = self.roles[slot as usize] == TextureRole::Distortion && !texture.has_blue();
    self.states[slot as usize] = RenderTextureState::Loaded {
      width: texture.width,
      height: texture.height,
      levels: texture.levels.len() as u32,
      layout: texture.layout.clone(),
      is_expanded: texture.is_expanded,
    };
    self.is_dirty = true;
  }

  /// Stands something in for a texture that cannot be had: a checker for a base, so the gap is seen, else neutral.
  fn stand_in(&mut self, slot: u32, state: RenderTextureState) {
    self.blue_free[slot as usize] = false;

    if self.roles[slot as usize] == TextureRole::Base {
      self.views[slot as usize] = self.checker.clone();
      self.is_dirty = true;
    }

    self.states[slot as usize] = state;
  }
}
