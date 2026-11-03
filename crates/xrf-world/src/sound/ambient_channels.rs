use std::collections::HashMap;
use std::f32::consts::TAU;
use std::time::Duration;

use glam::Vec3;

use crate::contract::sound::world_sound_channel_report::WorldSoundChannelReport;
use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::world_sound::WorldSound;
use crate::host::sound::world_sound_channel::WorldSoundChannel;
use crate::host::world_level_weather::WorldLevelWeather;
use crate::sound::sound_emitters::{SoundEmitters, SoundPlay};
use crate::sound::sound_random::SoundRandom;
use crate::weather::weather_ambients::WeatherAmbients;

/// What the channels' schedule draws from on every level open.
const SEED: u64 = 0xC4A7_7E15;

/// Metres above the camera a channel's sound plays.
const HEIGHT: f32 = 10.0;

/// The weather ambient's sound channels as `CGamePersistent::WeathersUpdate` plays them, indoors too: each channel,
/// by its place in the ambient's list, first waits a drawn time, then plays one of its sounds at random around the
/// camera, a drawn distance out on the level and ten metres up, and waits the sound's length and a drawn pause before
/// the next. The ambient is drawn each update between the two keyframes' by how far the weather stands between them.
pub struct AmbientChannels {
  /// `ambient_sound_next_time`, by the channel's place, which an ambient taking over keeps.
  next: Vec<Option<Duration>>,
  /// Each channel's sounds' last plays, which a sound picked again while playing starts over.
  playing: HashMap<(String, usize), u64>,
  /// The ambient the last update played.
  ambient: Option<String>,
  random: SoundRandom,
}

impl Default for AmbientChannels {
  fn default() -> Self {
    Self {
      next: Vec::new(),
      playing: HashMap::new(),
      ambient: None,
      random: SoundRandom::new(SEED),
    }
  }
}

impl AmbientChannels {
  /// Plays on to `now`, the level's clock, the camera standing at `eye` in engine space.
  pub fn update(
    &mut self,
    emitters: &mut SoundEmitters,
    level: &WorldLevelWeather,
    ambients: &WeatherAmbients,
    eye: Vec3,
    now: Duration,
  ) {
    let name: Option<&str> = if ambients.is_settled() {
      ambients.names[0].as_deref()
    } else {
      ambients.pick(self.random.fraction())
    };

    self.ambient = name.map(str::to_owned);

    let Some(ambient) = name.and_then(|name| level.ambients.get(name)) else {
      return;
    };

    if self.next.len() < ambient.sound_channels.len() {
      self.next.resize(ambient.sound_channels.len(), None);
    }

    for (index, channel_name) in ambient.sound_channels.iter().enumerate() {
      let Some(channel) = level.sound_channels.get(channel_name) else {
        continue;
      };

      match self.next[index] {
        None => self.next[index] = Some(now + Self::draw_period(&mut self.random, channel.period, 0)),
        Some(next) if now > next => {
          let sound_index: usize = self.random.index(channel.sounds.len());
          let Some(sound) = channel.sounds.get(sound_index) else {
            continue;
          };
          let position: Vec3 = self.draw_position(channel, eye);
          let key: (String, usize) = (channel_name.clone(), sound_index);

          if let Some(previous) = self.playing.remove(&key) {
            emitters.stop(previous);
          }

          if let Some(id) = emitters.play(
            level.engine,
            SoundPlay::new(sound, WorldSoundKind::Channel, position.to_array(), now),
          ) {
            self.playing.insert(key, id);
          }

          self.next[index] =
            Some(now + Self::get_length(sound) + Self::draw_period(&mut self.random, channel.period, 2));
        }
        Some(_) => {}
      }
    }

    self.playing.retain(|_, id| emitters.is_playing(*id, now));
  }

  /// The ambient the last update played, none while the weather names none.
  pub fn get_ambient(&self) -> Option<&str> {
    self.ambient.as_deref()
  }

  /// How long until each channel of the ambient the last update played sounds next.
  pub fn report(&self, level: Option<&WorldLevelWeather>, now: Duration) -> Vec<WorldSoundChannelReport> {
    let Some(ambient) = level
      .zip(self.ambient.as_deref())
      .and_then(|(level, name)| level.ambients.get(name))
    else {
      return Vec::new();
    };

    ambient
      .sound_channels
      .iter()
      .enumerate()
      .map(|(index, channel)| WorldSoundChannelReport {
        channel: channel.clone(),
        wait: self
          .next
          .get(index)
          .copied()
          .flatten()
          .map_or(Duration::ZERO, |next| next.saturating_sub(now)),
      })
      .collect()
  }

  /// `get_rnd_sound_first_time` from `first`, `get_rnd_sound_time` from `first + 2`: drawn where the range is not
  /// empty, else nought.
  fn draw_period(random: &mut SoundRandom, period: [u32; 4], first: usize) -> Duration {
    let (least, most) = (period[first], period[first + 1]);

    if least < most {
      random.millis([least, most])
    } else {
      Duration::ZERO
    }
  }

  /// A random direction on the level, a drawn distance out from the camera, and ten metres up.
  fn draw_position(&mut self, channel: &WorldSoundChannel, eye: Vec3) -> Vec3 {
    let angle: f32 = self.random.between(0.0, TAU);
    let [near, far] = channel.distance;
    let distance: f32 = if near < far {
      self.random.between(near, far)
    } else {
      0.0
    };

    eye + Vec3::new(angle.cos() * distance, HEIGHT, angle.sin() * distance)
  }

  /// The sound's length in whole milliseconds, `iFloor(get_length_sec() * 1000)`; nought for a missing file.
  fn get_length(sound: &WorldSound) -> Duration {
    sound.info.map_or(Duration::ZERO, |info| {
      Duration::from_millis(info.length.as_millis() as u64)
    })
  }
}
