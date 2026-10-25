use std::time::Instant;

use xrf_app_core::assets::read_located_asset;
use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_light_anim::LightAnimFile;
use xrf_ltx::Ltx;
use xrf_material::XrayTextureScope;
use xrf_vfs::XraySearch;
use xrf_visual::{LightsDescription, LightsPacker};

use crate::level::SelectedLevel;
use crate::parts::LevelTextureReference;
use crate::parts::{LevelSpawnPart, PackedLevelLights, SpawnVisualReader};
use crate::report::LevelReport;

/// What a change event calls what a level's lights were read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_LIGHTS_PART: &str = "lights";

/// What a level's lights were read from, those its spawned objects carry and its own: the colour animations and the
/// projectors' textures with the parts they ask for; the renderer holds the lights themselves.
pub(crate) struct LevelLightsPart;

impl SessionPart for LevelLightsPart {
  type Value = ();

  const NAME: &'static str = LEVEL_LIGHTS_PART;

  const IS_POOLED: bool = false;
}

impl LevelLightsPart {
  /// The colour animations lights name.
  pub const ANIMATIONS_FILE: &'static str = "lanims.xr";

  /// Collects the open level's lights, what they are read from recorded as its lights part. Where its spawn cannot be
  /// read the level keeps its own lights alone, and without configs its zones light nothing and its lamps keep their
  /// spawned flags.
  pub fn pack(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    sections: Option<&Ltx>,
  ) -> PackedLevelLights {
    current
      .parts
      .record_read::<Self, _>(search, |search| Self::pack_recorded(current, search, sections))
  }

  /// The lights packed, every read through `search`.
  fn pack_recorded(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    sections: Option<&Ltx>,
  ) -> PackedLevelLights {
    let started: Instant = Instant::now();
    let animations: Option<LightAnimFile> = read_located_asset(search, Self::ANIMATIONS_FILE)
      .and_then(LightAnimFile::read_from_bytes::<XRayByteOrder>)
      .inspect_err(|error| LevelReport::unreadable_animations(Self::ANIMATIONS_FILE, error))
      .ok();
    let mut packer: LightsPacker = LightsPacker::new(animations.as_ref());

    if let Some(sections) = sections {
      packer = packer.with_sections(sections);
    }

    match LevelSpawnPart::get(current, search) {
      Ok(spawn) => SpawnVisualReader::with(current, search, |visuals| {
        packer.add_objects(spawn.list_kept(), &mut |name| {
          visuals.get(name).ok().and_then(|it| it.rest.clone())
        });
      }),
      Err(error) => LevelReport::missing_spawned_lights(&current.source, &error),
    }

    if let Some(lights) = current.level.lights.as_ref() {
      packer.add_level_lights(&lights.lights);
    }

    let lights: LightsDescription = packer.pack();
    let scope: XrayTextureScope = current.source.get_texture_scope();

    LevelReport::lights(&current.source, &lights, started);

    PackedLevelLights {
      projectors: lights
        .projectors
        .iter()
        .map(|reference| LevelTextureReference::locate(&scope, search, reference.clone()))
        .collect(),
      lights,
    }
  }
}
