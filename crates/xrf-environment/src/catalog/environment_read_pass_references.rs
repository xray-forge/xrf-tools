//! What an environment read checks once everything is read: that each name a section writes names a definition the
//! engine finds, taken from `system.ltx` where OpenXRay would.

use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_ltx::Section;

use crate::ambient::{Ambient, AmbientEffect, AmbientKey, SoundChannel};
use crate::catalog::environment_named::EnvironmentNamed;
use crate::catalog::environment_read_pass::EnvironmentReadPass;
use crate::finding::{EnvironmentFinding, EnvironmentRule};
use crate::section::EnvironmentSectionReader;
use crate::sun::{LensFlare, LensFlareKey};
use crate::thunderbolt::{Thunderbolt, ThunderboltCollection, ThunderboltKey};
use crate::weather::{WeatherCycle, WeatherKey};

impl EnvironmentReadPass<'_> {
  /// Checks the sun, collection and ambient a keyframe names, taking a sun or collection OpenXRay would find in
  /// `system.ltx` from there.
  pub(super) fn resolve_keyframe_references(
    &mut self,
    cycle: &WeatherCycle,
    suns: &mut Vec<LensFlare>,
    collections: &mut Vec<ThunderboltCollection>,
    ambients: &[Ambient],
  ) -> XrfResult<()> {
    for keyframe in &cycle.keyframes {
      let section = &keyframe.section;
      let sun: &str = section.get_text(WeatherKey::Sun, self.options.engine);
      let collection: &str = section.get_text(WeatherKey::ThunderboltCollection, self.options.engine);
      let ambient: &str = section.get_text(WeatherKey::Ambient, self.options.engine);

      if !sun.is_empty() && !suns.iter().any(|it| it.name == sun) && !self.take_system_sun(sun, suns)? {
        self.report_reference(cycle, &section.name, "sun", format!("sun [{sun}]"));
      }

      if !collection.is_empty()
        && !collections.iter().any(|it| it.name == collection)
        && !self.take_system_collection(collection, collections)?
      {
        let target: String = format!("thunderbolt collection [{collection}]");

        self.report_reference(cycle, &section.name, "thunderbolt_collection", target);
      }

      if !ambient.is_empty() && !ambients.iter().any(|it| it.name == ambient) {
        self.report_reference(cycle, &section.name, "ambient", format!("ambient [{ambient}]"));
      }
    }

    Ok(())
  }

  /// Checks each collection's bolts: in `thunderbolts.ltx` for a collection of `thunderbolt_collections.ltx`, in
  /// `system.ltx` for one OpenXRay took from there.
  pub(super) fn resolve_collection_references(
    &mut self,
    collections: &[ThunderboltCollection],
    thunderbolts: &mut Vec<Thunderbolt>,
  ) -> XrfResult<()> {
    let system: Option<String> = self
      .project
      .config_path(Self::SYSTEM)
      .ok()
      .map(|path| path.as_str().to_owned());

    for collection in collections {
      let is_system: bool = system.as_deref() == Some(collection.file.as_str());

      for bolt in &collection.thunderbolts {
        let is_found: bool = match is_system {
          true => self.take_system_section(bolt, "Thunderbolt", thunderbolts, |reader, name, section| {
            reader.read::<ThunderboltKey>(name, section)
          })?,
          false => thunderbolts.iter().any(|it| it.name == *bolt),
        };

        if !is_found {
          let message: String = format!(
            "Thunderbolt collection [{}] references missing thunderbolt [{bolt}]",
            collection.name
          );

          self.push_reference(&collection.file, &collection.name, None, message);
        }
      }
    }

    Ok(())
  }

  pub(super) fn resolve_ambient_references(
    &mut self,
    ambient: &Ambient,
    channels: &[SoundChannel],
    effects: &[AmbientEffect],
  ) {
    if !AmbientKey::is_own_channel(ambient, self.options.engine) {
      for channel in AmbientKey::list_channels(ambient) {
        if !channel.is_empty() && !channels.iter().any(|it| it.name == *channel) {
          let message: String = format!(
            "Ambient [{}] references missing sound channel [{channel}]",
            ambient.name
          );

          self.push_reference(&ambient.file, &ambient.name, Some("sound_channels"), message);
        }
      }
    }

    for effect in ambient.get_list(AmbientKey::Effects) {
      if !effect.is_empty() && !effects.iter().any(|it| it.name == *effect) {
        let message: String = format!("Ambient [{}] references missing effect [{effect}]", ambient.name);

        self.push_reference(&ambient.file, &ambient.name, Some("effects"), message);
      }
    }
  }

  /// On OpenXRay, a sun `system.ltx` holds in place of `suns.ltx`, added to the suns; whether one was found.
  pub(super) fn take_system_sun(&mut self, name: &str, suns: &mut Vec<LensFlare>) -> XrfResult<bool> {
    self.take_system_section(name, "Sun", suns, LensFlareKey::read)
  }

  /// On OpenXRay, a collection `system.ltx` holds in place of `thunderbolt_collections.ltx`; whether one was found.
  pub(super) fn take_system_collection(
    &mut self,
    name: &str,
    collections: &mut Vec<ThunderboltCollection>,
  ) -> XrfResult<bool> {
    self.take_system_section(name, "Thunderbolt collection", collections, ThunderboltCollection::read)
  }

  /// Reads one `system.ltx` section into a list, on OpenXRay only, unless it is there already.
  pub(super) fn take_system_section<T>(
    &mut self,
    name: &str,
    subject: &'static str,
    list: &mut Vec<T>,
    read: impl Fn(&mut EnvironmentSectionReader, &str, &Section) -> T,
  ) -> XrfResult<bool>
  where
    T: EnvironmentNamed,
  {
    if self.options.engine != XrayEngine::Vanilla {
      return Ok(false);
    }

    let Some(system) = self.get_system()? else {
      return Ok(false);
    };

    if list
      .iter()
      .any(|it| it.get_name() == name && it.get_file() == system.file)
    {
      return Ok(true);
    }

    let Some(section) = system.get_ltx().section(name) else {
      return Ok(false);
    };
    let mut reader: EnvironmentSectionReader = self.reader(subject, &system);

    list.push(read(&mut reader, name, section));

    Ok(true)
  }

  pub(super) fn report_reference(&mut self, cycle: &WeatherCycle, section: &str, key: &str, target: String) {
    let message: String = format!("{} [{section}] references missing {target}", cycle.kind.get_subject());
    let file: &str = &cycle.file;

    self.push_reference(file, section, Some(key), message);
  }

  pub(super) fn push_reference(&mut self, file: &str, section: &str, key: Option<&str>, message: String) {
    self.findings.push(EnvironmentFinding {
      file: file.to_owned(),
      key: key.map(str::to_owned),
      message,
      rule: EnvironmentRule::Reference,
      section: Some(section.to_owned()),
    });
  }
}
