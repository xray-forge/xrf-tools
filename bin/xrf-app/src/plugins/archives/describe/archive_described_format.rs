use xrf_error::XrfResult;
use xrf_extension::{XrayExtension, XrayExtensionOf};
use xrf_vfs::XrayLogicalPath;

use crate::plugins::archives::describe::anm::ArchiveAnmDescription;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_file_description::ArchiveFormatDescription;
use crate::plugins::archives::describe::detail::{ArchiveDetailLibraryDescription, ArchiveDetailModel};
use crate::plugins::archives::describe::efd::ArchiveEfdDescription;
use crate::plugins::archives::describe::gamemtl::ArchiveGameMtlDescription;
use crate::plugins::archives::describe::level::{
  ArchiveLevelAiDescription, ArchiveLevelCollisionDescription, ArchiveLevelDescription, ArchiveLevelEnvModDescription,
  ArchiveLevelFogVolDescription, ArchiveLevelGameDescription, ArchiveLevelGeomDescription, ArchiveLevelHomDescription,
  ArchiveLevelLightsDescription, ArchiveLevelPsStaticDescription, ArchiveLevelSndStaticDescription,
  ArchiveLevelSomDescription, ArchiveLevelWallmarksDescription,
};
use crate::plugins::archives::describe::light_anim::ArchiveLightAnimDescription;
use crate::plugins::archives::describe::omf::ArchiveOmfDescription;
use crate::plugins::archives::describe::particles::ArchiveParticlesDescription;
use crate::plugins::archives::describe::ppe::ArchivePpeDescription;
use crate::plugins::archives::describe::shader_compiler::ArchiveShaderCompilerDescription;
use crate::plugins::archives::describe::shaders::ArchiveShadersDescription;
use crate::plugins::archives::describe::sound::ArchiveSoundEnvironmentDescription;
use crate::plugins::archives::describe::spawn::{ArchiveLevelSpawnDescription, ArchiveSpawnDescription};
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// Which describer answers for an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDescribedFormat {
  /// An object motion, `.anm` and `.anm1`, which the two spellings of share a format.
  Anm,
  /// A standalone detail object, `.dm`.
  Detail,
  /// A level's detail layer, `level.details`.
  DetailLibrary,
  /// A trained evaluation function, `.efd`.
  Efd,
  /// A level's local weather overrides, `level.env_mod`.
  LevelEnvMod,
  /// A level's volumetric fog, `level.fog_vol`.
  LevelFogVol,
  /// A level's respawn points and patrol paths, `level.game`.
  LevelGame,
  /// The game material library, `gamemtl.xr`.
  GameMtl,
  /// A level's occlusion mesh, `level.hom`.
  LevelHom,
  /// A level's render geometry, `level.geom`.
  LevelGeom,
  /// A level's detail render geometry, `level.geomX`, which is the same format drawn at distance.
  LevelGeomDetail,
  /// The colour animation library, `lanims.xr`.
  LightAnim,
  /// The compiler shader library, `shaders_xrlc.xr`.
  ShaderCompiler,
  /// The sound environment library, `senvironment.xr`.
  SoundEnvironment,
  /// A level's compiled lights, `build.lights`.
  LevelLights,
  /// The particle effects a level plants, `level.ps_static`.
  LevelPsStatic,
  /// The sounds a level plants, `level.snd_static`.
  LevelSndStatic,
  /// A level's sound occlusion mesh, `level.som`.
  LevelSom,
  /// A level's baked decals, `level.wallmarks`.
  LevelWallmarks,
  /// A level's navigation grid, `level.ai`.
  LevelAi,
  /// A level's collision mesh, `level.cform`.
  LevelCollision,
  /// A compiled level bundle, `level`.
  Level,
  /// A motion bank, `CKinematicsAnimated`'s partition and motions.
  Omf,
  /// The particle library, `particles.xr`.
  Particles,
  /// A post-process effect, `.ppe`.
  Ppe,
  /// The compiled blender library, `shaders.xr`.
  Shaders,
  /// Either format under `.spawn`: a set where the file carries a header, and a level's object list where it does
  /// not. Which of the two an entry is cannot be told from its name, so the describer looks.
  Spawn,
  /// A texture descriptor, `ETextureThumbnail` wrapping `STextureParams`.
  Thm,
}

impl ArchiveDescribedFormat {
  /// Files the engine loads by a name of their own rather than by a kind, folded as engine paths.
  ///
  /// Three reasons land here. `.xr` says only that a file is a chunked X-Ray library, and six unrelated formats share
  /// it; `level` has no extension at all; `.details` is a real format, but vanilla also ships a compiler intermediate
  /// under it, so only the file called `level.details` is the detail layer. Either way the name the engine loads the
  /// file under is the whole of the answer. All six `.xr` libraries are named here now, so the extension has no
  /// unclaimed spelling left in the trees.
  const NAMED: [(&'static str, Self); 8] = [
    ("gamemtl.xr", Self::GameMtl),
    ("lanims.xr", Self::LightAnim),
    ("level", Self::Level),
    ("level.details", Self::DetailLibrary),
    ("particles.xr", Self::Particles),
    ("senvironment.xr", Self::SoundEnvironment),
    ("shaders.xr", Self::Shaders),
    ("shaders_xrlc.xr", Self::ShaderCompiler),
  ];

  /// The describer for an entry name, or `None` when no describer claims it.
  pub fn of(name: &str) -> Option<Self> {
    match XrayExtensionOf::of(name).known() {
      Some(XrayExtension::Anm | XrayExtension::Anm1) => Some(Self::Anm),
      Some(XrayExtension::EnvMod) => Some(Self::LevelEnvMod),
      Some(XrayExtension::FogVol) => Some(Self::LevelFogVol),
      Some(XrayExtension::Game) => Some(Self::LevelGame),
      Some(XrayExtension::Geom) => Some(Self::LevelGeom),
      Some(XrayExtension::GeomX) => Some(Self::LevelGeomDetail),
      Some(XrayExtension::Hom) => Some(Self::LevelHom),
      Some(XrayExtension::Lights) => Some(Self::LevelLights),
      Some(XrayExtension::PsStatic) => Some(Self::LevelPsStatic),
      Some(XrayExtension::SndStatic) => Some(Self::LevelSndStatic),
      Some(XrayExtension::Som) => Some(Self::LevelSom),
      Some(XrayExtension::Wallmarks) => Some(Self::LevelWallmarks),
      Some(XrayExtension::Efd) => Some(Self::Efd),
      Some(XrayExtension::Ai) => Some(Self::LevelAi),
      Some(XrayExtension::CForm) => Some(Self::LevelCollision),
      Some(XrayExtension::Dm) => Some(Self::Detail),
      Some(XrayExtension::Omf) => Some(Self::Omf),
      Some(XrayExtension::Ppe) => Some(Self::Ppe),
      Some(XrayExtension::Spawn) => Some(Self::Spawn),
      Some(XrayExtension::Thm) => Some(Self::Thm),
      // An extension that names a container rather than a format, and a name carrying none at all, ask the same
      // question: which file is this, by the name the engine loads it under. `.details` is here for the narrower
      // version of the same reason: vanilla ships one that is a compiler intermediate, so only `level.details` is
      // the detail layer.
      Some(XrayExtension::Details | XrayExtension::Xr) | None => Self::of_named(name),
      Some(_) => None,
    }
  }

  /// Whether this describer reads by seeking rather than by holding the entry, and so answers to no size ceiling.
  pub const fn reads_by_seeking(self) -> bool {
    matches!(
      self,
      Self::Spawn | Self::LevelAi | Self::LevelCollision | Self::LevelGeom | Self::LevelGeomDetail
    )
  }

  /// Reads the entry this format claimed, or answers `None` where the claim does not hold after all.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not the format this claimed them as.
  pub fn describe(self, source: &ArchiveDescribeSource, name: &str) -> XrfResult<Option<ArchiveFormatDescription>> {
    Ok(match self {
      Self::Anm => Some(ArchiveFormatDescription::Anm {
        description: Box::new(ArchiveAnmDescription::read(source, name)?),
      }),
      Self::Detail => Some(ArchiveFormatDescription::Detail {
        description: Box::new(ArchiveDetailModel::read(source, name)?),
      }),
      Self::DetailLibrary => Some(ArchiveFormatDescription::DetailLibrary {
        description: Box::new(ArchiveDetailLibraryDescription::read(source, name)?),
      }),
      Self::LevelEnvMod => Some(ArchiveFormatDescription::LevelEnvMod {
        description: Box::new(ArchiveLevelEnvModDescription::read(source, name)?),
      }),
      Self::LevelFogVol => Some(ArchiveFormatDescription::LevelFogVol {
        description: Box::new(ArchiveLevelFogVolDescription::read(source, name)?),
      }),
      Self::LevelGame => Some(ArchiveFormatDescription::LevelGame {
        description: Box::new(ArchiveLevelGameDescription::read(source, name)?),
      }),
      Self::GameMtl => Some(ArchiveFormatDescription::GameMtl {
        description: Box::new(ArchiveGameMtlDescription::read(source, name)?),
      }),
      Self::LevelHom => Some(ArchiveFormatDescription::LevelHom {
        description: Box::new(ArchiveLevelHomDescription::read(source, name)?),
      }),
      Self::LevelGeom => Some(ArchiveFormatDescription::LevelGeom {
        description: Box::new(ArchiveLevelGeomDescription::read(source, name, false)?),
      }),
      Self::LevelGeomDetail => Some(ArchiveFormatDescription::LevelGeom {
        description: Box::new(ArchiveLevelGeomDescription::read(source, name, true)?),
      }),
      Self::LightAnim => Some(ArchiveFormatDescription::LightAnim {
        description: Box::new(ArchiveLightAnimDescription::read(source, name)?),
      }),
      Self::ShaderCompiler => Some(ArchiveFormatDescription::ShaderCompiler {
        description: Box::new(ArchiveShaderCompilerDescription::read(source, name)?),
      }),
      Self::SoundEnvironment => Some(ArchiveFormatDescription::SoundEnvironment {
        description: Box::new(ArchiveSoundEnvironmentDescription::read(source, name)?),
      }),
      Self::LevelLights => Some(ArchiveFormatDescription::LevelLights {
        description: Box::new(ArchiveLevelLightsDescription::read(source, name)?),
      }),
      Self::LevelPsStatic => Some(ArchiveFormatDescription::LevelPsStatic {
        description: Box::new(ArchiveLevelPsStaticDescription::read(source, name)?),
      }),
      Self::LevelSndStatic => Some(ArchiveFormatDescription::LevelSndStatic {
        description: Box::new(ArchiveLevelSndStaticDescription::read(source, name)?),
      }),
      Self::LevelSom => Some(ArchiveFormatDescription::LevelSom {
        description: Box::new(ArchiveLevelSomDescription::read(source, name)?),
      }),
      Self::LevelWallmarks => Some(ArchiveFormatDescription::LevelWallmarks {
        description: Box::new(ArchiveLevelWallmarksDescription::read(source, name)?),
      }),
      Self::Efd => Some(ArchiveFormatDescription::Efd {
        description: Box::new(ArchiveEfdDescription::read(source, name)?),
      }),
      Self::LevelAi => Some(ArchiveFormatDescription::LevelAi {
        description: Box::new(ArchiveLevelAiDescription::read(source, name)?),
      }),
      Self::LevelCollision => Some(ArchiveFormatDescription::LevelCollision {
        description: Box::new(ArchiveLevelCollisionDescription::read(source, name)?),
      }),
      Self::Level => Some(ArchiveFormatDescription::Level {
        description: Box::new(ArchiveLevelDescription::read(source, name)?),
      }),
      Self::Omf => Some(ArchiveFormatDescription::Omf {
        description: Box::new(ArchiveOmfDescription::read(source, name)?),
      }),
      Self::Particles => Some(ArchiveFormatDescription::Particles {
        description: Box::new(ArchiveParticlesDescription::read(source, name)?),
      }),
      Self::Ppe => Some(ArchiveFormatDescription::Ppe {
        description: Box::new(ArchivePpeDescription::read(source, name)?),
      }),
      Self::Shaders => Some(ArchiveFormatDescription::Shaders {
        description: Box::new(ArchiveShadersDescription::read(source, name)?),
      }),
      Self::Spawn => match ArchiveSpawnDescription::read(source, name)? {
        Some(description) => Some(ArchiveFormatDescription::Spawn {
          description: Box::new(description),
        }),
        // Not a set, so try the other format under this extension before giving the entry up to the container walk.
        None => {
          ArchiveLevelSpawnDescription::read(source, name)?.map(|description| ArchiveFormatDescription::LevelSpawn {
            description: Box::new(description),
          })
        }
      },
      Self::Thm => Some(ArchiveFormatDescription::Thm {
        description: Box::new(ArchiveThmDescription::read(source, name)?),
      }),
    })
  }

  /// Which named file an entry is, folded the way the engine addresses it.
  ///
  /// Through [`XrayLogicalPath`] rather than by slicing the name, so the case and separator rule is the one the name
  /// table already agrees on and is not restated here - which is the whole reason this decision is not the
  /// frontend's to make.
  fn of_named(name: &str) -> Option<Self> {
    let path: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;
    let file: &str = path.file_name();

    Self::NAMED
      .into_iter()
      .find_map(|(candidate, format)| (candidate == file).then_some(format))
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveDescribedFormat;

  #[test]
  fn a_descriptor_is_claimed_however_its_extension_is_spelled() {
    assert_eq!(
      ArchiveDescribedFormat::of("textures\\act\\act_arm_1.thm"),
      Some(ArchiveDescribedFormat::Thm)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("textures\\act\\act_arm_1.THM"),
      Some(ArchiveDescribedFormat::Thm)
    );
  }

  #[test]
  fn a_motion_bank_is_claimed_however_its_extension_is_spelled() {
    assert_eq!(
      ArchiveDescribedFormat::of("meshes\\actors\\stalker_animation.omf"),
      Some(ArchiveDescribedFormat::Omf)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("meshes\\actors\\stalker_animation.OMF"),
      Some(ArchiveDescribedFormat::Omf)
    );
  }

  #[test]
  fn an_object_motion_is_claimed_under_either_spelling_of_its_extension() {
    for name in [
      "anims\\camera_effects\\head_shot.anm",
      "anims\\camera_effects\\head_shot.ANM",
      "anims\\dof_zoom_in.anm1",
    ] {
      assert_eq!(
        ArchiveDescribedFormat::of(name),
        Some(ArchiveDescribedFormat::Anm),
        "'{name}' is an object motion"
      );
    }
  }

  #[test]
  fn a_post_process_effect_is_claimed_by_its_extension() {
    assert_eq!(
      ArchiveDescribedFormat::of("anims\\camera_effects\\acidic.ppe"),
      Some(ArchiveDescribedFormat::Ppe)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("anims\\BLINK.PPE"),
      Some(ArchiveDescribedFormat::Ppe)
    );
  }

  #[test]
  fn a_detail_layer_is_claimed_by_its_name_and_a_detail_object_by_its_extension() {
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\level.details"),
      Some(ArchiveDescribedFormat::DetailLibrary)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("meshes\\dm\\rain.dm"),
      Some(ArchiveDescribedFormat::Detail)
    );
  }

  #[test]
  fn a_file_that_only_shares_the_detail_extension_is_left_to_the_container_walk() {
    // Vanilla ships `recalculation_data_slots.details` beside a real one, and it is a different format: a four byte
    // first chunk where a library has a twenty-four byte header. Keying the layer by name is what keeps it out.
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\mp_pripyat\\recalculation_data_slots.details"),
      None
    );
  }

  #[test]
  fn every_level_companion_is_claimed_by_its_extension() {
    for (name, expected) in [
      ("levels\\l01_escape\\level.hom", ArchiveDescribedFormat::LevelHom),
      ("levels\\l01_escape\\level.som", ArchiveDescribedFormat::LevelSom),
      ("levels\\l01_escape\\level.env_mod", ArchiveDescribedFormat::LevelEnvMod),
      ("levels\\l01_escape\\level.fog_vol", ArchiveDescribedFormat::LevelFogVol),
      (
        "levels\\l01_escape\\level.ps_static",
        ArchiveDescribedFormat::LevelPsStatic,
      ),
      (
        "levels\\l01_escape\\level.snd_static",
        ArchiveDescribedFormat::LevelSndStatic,
      ),
      ("levels\\l01_escape\\level.game", ArchiveDescribedFormat::LevelGame),
      (
        "levels\\l01_escape\\level.wallmarks",
        ArchiveDescribedFormat::LevelWallmarks,
      ),
    ] {
      assert_eq!(
        ArchiveDescribedFormat::of(name),
        Some(expected),
        "'{name}' is a level companion"
      );
    }
  }

  #[test]
  fn a_companion_the_compiler_names_for_itself_is_still_claimed() {
    // The light list is `build.lights`, not `level.lights`, and the two atmosphere files also ship disabled under a
    // renamed stem. Keying by extension claims all three where keying by file name would miss them.
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\build.lights"),
      Some(ArchiveDescribedFormat::LevelLights)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\_disabled_level.env_mod"),
      Some(ArchiveDescribedFormat::LevelEnvMod)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\_disabled_level.fog_vol"),
      Some(ArchiveDescribedFormat::LevelFogVol)
    );
  }

  #[test]
  fn an_evaluation_function_is_claimed_wherever_it_sits() {
    // Unlike the companions these carry no shared stem at all: every one is named for what it evaluates.
    assert_eq!(
      ArchiveDescribedFormat::of("common\\WeaponEffectiveness.efd"),
      Some(ArchiveDescribedFormat::Efd)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("common\\birthpercentage.EFD"),
      Some(ArchiveDescribedFormat::Efd)
    );
  }

  #[test]
  fn a_library_is_claimed_by_its_stem_rather_than_by_its_extension() {
    assert_eq!(
      ArchiveDescribedFormat::of("shaders.xr"),
      Some(ArchiveDescribedFormat::Shaders)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("Particles.XR"),
      Some(ArchiveDescribedFormat::Particles)
    );
  }

  #[test]
  fn a_library_is_claimed_wherever_it_sits() {
    // The libraries ship at the root of a tree, but a name table addresses an entry by its whole path and an archive
    // is free to hold one deeper.
    assert_eq!(
      ArchiveDescribedFormat::of("mod\\overrides/Shaders.xr"),
      Some(ArchiveDescribedFormat::Shaders)
    );
  }

  #[test]
  fn a_library_no_name_claims_stays_unclaimed_though_it_shares_the_extension() {
    // The four libraries this used to pin as unclaimed now have readers. What it guards is unchanged: `.xr` claims
    // nothing by its extension, so a library nobody named falls through rather than being read as another one.
    for name in ["unknown.xr", "shaders_xrlc_backup.xr", "gamemtl2.xr"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' has no describer yet");
    }
  }

  #[test]
  fn a_stem_that_only_ends_in_a_library_name_is_not_one() {
    for name in ["my_shaders.xr", "shaders_backup.xr", "particles2.xr"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' is not a library");
    }
  }

  #[test]
  fn a_bundle_is_claimed_though_its_name_carries_no_extension() {
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\level"),
      Some(ArchiveDescribedFormat::Level)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("Levels/L01_Escape/LEVEL"),
      Some(ArchiveDescribedFormat::Level)
    );
  }

  #[test]
  fn a_name_that_only_begins_with_a_named_file_is_not_one() {
    // `level.ai` is claimed, and as a different format: the bundle is the file called `level` and nothing else, which
    // is the whole point of matching a file name rather than a prefix of one.
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\level.ai"),
      Some(ArchiveDescribedFormat::LevelAi)
    );

    for name in ["levels\\l01_escape\\level_lods", "mylevel", "levelx"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' is not a bundle");
    }
  }

  #[test]
  fn render_geometry_and_its_detail_twin_are_told_apart_by_their_extensions() {
    // The same reader answers for both, but a description says which it read: `level.geomX` is the geometry the
    // renderer draws at distance and carries no progressive meshes at all.
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\level.geom"),
      Some(ArchiveDescribedFormat::LevelGeom)
    );

    for name in ["levels\\l01_escape\\level.geomx", "levels\\l01_escape\\level.geomX"] {
      assert_eq!(
        ArchiveDescribedFormat::of(name),
        Some(ArchiveDescribedFormat::LevelGeomDetail),
        "'{name}' is detail geometry"
      );
    }
  }

  #[test]
  fn render_geometry_is_read_by_seeking_rather_than_held() {
    // A `level.geom` reaches 143 MB, so a describer answering to the size ceiling would refuse most of them.
    for format in [
      ArchiveDescribedFormat::LevelGeom,
      ArchiveDescribedFormat::LevelGeomDetail,
    ] {
      assert!(format.reads_by_seeking(), "{format:?} is too large to hold whole");
    }
  }

  #[test]
  fn every_xr_library_is_claimed_by_its_own_name() {
    // `.xr` names a container rather than a format, so each library is claimed by the name the engine loads it under.
    for (name, expected) in [
      ("gamemtl.xr", ArchiveDescribedFormat::GameMtl),
      ("lanims.xr", ArchiveDescribedFormat::LightAnim),
      ("particles.xr", ArchiveDescribedFormat::Particles),
      ("senvironment.xr", ArchiveDescribedFormat::SoundEnvironment),
      ("shaders.xr", ArchiveDescribedFormat::Shaders),
      ("shaders_xrlc.xr", ArchiveDescribedFormat::ShaderCompiler),
    ] {
      assert_eq!(
        ArchiveDescribedFormat::of(name),
        Some(expected),
        "'{name}' is a library"
      );
    }
  }

  #[test]
  fn the_two_shader_libraries_are_not_the_same_format() {
    // `shaders.xr` is the renderer's blenders and `shaders_xrlc.xr` is what the compiler was told; they share names
    // and nothing else, and the second is not even a chunk tree.
    assert_ne!(
      ArchiveDescribedFormat::of("shaders.xr"),
      ArchiveDescribedFormat::of("shaders_xrlc.xr")
    );
  }

  #[test]
  fn everything_without_a_describer_is_left_unclaimed() {
    for name in ["meshes\\actor.ogf", "textures\\act\\act_arm_1.dds", "menu\\intro.ogm"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' has no describer yet");
    }
  }
}
