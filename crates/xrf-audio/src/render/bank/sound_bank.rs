use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::decode::decoded_sound::DecodedSound;
use crate::decode::sound_decoder::SoundDecoder;
use crate::decode::sound_format::SoundFormat;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::render::bank::banked_sound::BankedSound;
use crate::render::bank::sound_key::SoundKey;

/// The sounds the scenes play, by their source's scope and reference, read and decoded off the control thread: on a
/// loader thread of its own, or in the call that asks for them in an offline session. A sound up to
/// [`Self::DECODED_SECONDS`] long is decoded once and shared by every voice playing it; a longer one is streamed from
/// its bytes. Past its byte budget the bank lets go of the sounds no emitter holds, the least recently asked first; one
/// asked for again is read again.
pub(crate) struct SoundBank {
  sounds: HashMap<SoundKey, BankEntry>,
  loader: SoundLoader,
  /// Bytes the bank may hold before it lets go of what no emitter holds.
  budget: usize,
  /// Bytes it holds.
  size: usize,
  /// Counts the asks, which date each sound's last.
  asks: u64,
}

struct BankEntry {
  sound: BankedSound,
  asked: u64,
}

/// Where the bank's reads run.
enum SoundLoader {
  Thread {
    requests: Sender<SoundRequest>,
    loaded: Receiver<SoundLoaded>,
  },
  Inline,
}

struct SoundRequest {
  key: SoundKey,
  assets: Arc<dyn AudioAssetSource>,
}

struct SoundLoaded {
  key: SoundKey,
  sound: BankedSound,
}

impl SoundBank {
  /// The longest sound decoded whole: anything longer, music above all, is streamed.
  const DECODED_SECONDS: f64 = 15.0;

  /// What a bank holds before it lets go of what no emitter holds: about 25 minutes of decoded mono at 44.1 kHz.
  const BUDGET: usize = 256 << 20;

  /// A bank reading on a thread of its own.
  pub(crate) fn threaded() -> Self {
    let (requests, received) = mpsc::channel::<SoundRequest>();
    let (sender, loaded) = mpsc::channel::<SoundLoaded>();
    let spawned = thread::Builder::new()
      .name(String::from("xrf-audio-load"))
      .spawn(move || {
        while let Ok(SoundRequest { key, assets }) = received.recv() {
          let sound: BankedSound = Self::load(assets.as_ref(), &key.reference);

          if sender.send(SoundLoaded { key, sound }).is_err() {
            return;
          }
        }
      });

    match spawned {
      Ok(_) => Self::with(SoundLoader::Thread { requests, loaded }, Self::BUDGET),
      Err(error) => {
        log::error!("The audio's loader could not start, sounds load on the control thread: {error}");

        Self::inline()
      }
    }
  }

  /// A bank reading in the call that asks, for an offline session.
  pub(crate) fn inline() -> Self {
    Self::with(SoundLoader::Inline, Self::BUDGET)
  }

  /// A bank reading in the call that asks, holding `budget` bytes before it lets go.
  #[cfg(test)]
  pub(crate) fn inline_within(budget: usize) -> Self {
    Self::with(SoundLoader::Inline, budget)
  }

  fn with(loader: SoundLoader, budget: usize) -> Self {
    Self {
      sounds: HashMap::new(),
      loader,
      budget,
      size: 0,
      asks: 0,
    }
  }

  /// Bytes it holds.
  #[cfg(test)]
  pub(crate) fn get_size(&self) -> usize {
    self.size
  }

  /// The sound `reference` of `assets`, whose scope is `scope`, asked for the first time it is named.
  pub(crate) fn get(
    &mut self,
    scope: &Arc<str>,
    assets: &Arc<dyn AudioAssetSource>,
    reference: &Arc<str>,
  ) -> BankedSound {
    let key: SoundKey = SoundKey {
      scope: Arc::clone(scope),
      reference: Arc::clone(reference),
    };

    self.asks += 1;

    if let Some(entry) = self.sounds.get_mut(&key) {
      entry.asked = self.asks;

      return entry.sound.clone();
    }

    let sound: BankedSound = match &self.loader {
      SoundLoader::Inline => Self::load(assets.as_ref(), reference),
      SoundLoader::Thread { requests, .. } => {
        let request: SoundRequest = SoundRequest {
          key: key.clone(),
          assets: Arc::clone(assets),
        };

        match requests.send(request) {
          Ok(()) => BankedSound::Loading,
          Err(_) => BankedSound::Refused(String::from("The audio's loader has ended")),
        }
      }
    };

    self.insert(key, sound.clone());
    self.trim();

    sound
  }

  /// Takes what the loader read since the last call, and lets go of what no emitter holds past the budget.
  pub(crate) fn collect(&mut self) {
    if let SoundLoader::Thread { loaded, .. } = &self.loader {
      let received: Vec<SoundLoaded> = loaded.try_iter().collect();

      for SoundLoaded { key, sound } in received {
        if let BankedSound::Refused(reason) = &sound {
          log::warn!("Sound '{}' cannot be played: {reason}", key.reference);
        }

        self.insert(key, sound);
      }
    }

    self.trim();
  }

  fn insert(&mut self, key: SoundKey, sound: BankedSound) {
    let entry: BankEntry = BankEntry {
      asked: self.asks,
      sound,
    };

    self.size += entry.sound.get_size();

    if let Some(replaced) = self.sounds.insert(key, entry) {
      self.size -= replaced.sound.get_size();
    }
  }

  /// Lets go of the sounds no emitter holds, the least recently asked first, until the bank is within its budget.
  fn trim(&mut self) {
    if self.size <= self.budget {
      return;
    }

    let mut unheld: Vec<SoundKey> = self
      .sounds
      .iter()
      .filter(|(_, entry)| entry.sound.is_unheld())
      .map(|(key, _)| key.clone())
      .collect();

    unheld.sort_unstable_by_key(|key| self.sounds[key].asked);

    for key in unheld {
      if self.size <= self.budget {
        break;
      }

      if let Some(entry) = self.sounds.remove(&key) {
        self.size -= entry.sound.get_size();
      }
    }
  }

  fn load(assets: &dyn AudioAssetSource, reference: &str) -> BankedSound {
    let bytes: Arc<[u8]> = match assets.read_sound(reference) {
      Ok(Some(bytes)) => Arc::from(bytes),
      Ok(None) => return BankedSound::Refused(String::from("The sound's file is missing")),
      Err(error) => return BankedSound::Refused(format!("The sound's file cannot be read: {error}")),
    };
    let decoder: SoundDecoder = match SoundDecoder::open(Arc::clone(&bytes)) {
      Ok(decoder) => decoder,
      Err(error) => return BankedSound::Refused(format!("The sound cannot be decoded: {error}")),
    };
    let format: SoundFormat = decoder.get_format();

    if format.get_duration().as_secs_f64() > Self::DECODED_SECONDS {
      return BankedSound::Streamed { bytes, format };
    }

    match DecodedSound::decode_from(decoder) {
      Ok(sound) => BankedSound::Decoded(Arc::new(sound)),
      Err(error) => BankedSound::Refused(format!("The sound cannot be decoded: {error}")),
    }
  }
}
