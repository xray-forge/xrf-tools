use serde::Serialize;

use crate::describe::anm::ArchiveAnmDescription;
use crate::describe::chunks::ArchiveChunksDescription;
use crate::describe::description::ArchiveDescribeRefusal;
use crate::describe::detail::{ArchiveDetailLibraryDescription, ArchiveDetailModel};
use crate::describe::efd::ArchiveEfdDescription;
use crate::describe::gamemtl::ArchiveGameMtlDescription;
use crate::describe::level::{
  ArchiveLevelAiDescription, ArchiveLevelCollisionDescription, ArchiveLevelDescription, ArchiveLevelEnvModDescription,
  ArchiveLevelFogVolDescription, ArchiveLevelGameDescription, ArchiveLevelGeomDescription, ArchiveLevelHomDescription,
  ArchiveLevelLightsDescription, ArchiveLevelPsStaticDescription, ArchiveLevelSndStaticDescription,
  ArchiveLevelSomDescription, ArchiveLevelWallmarksDescription,
};
use crate::describe::light_anim::ArchiveLightAnimDescription;
use crate::describe::omf::ArchiveOmfDescription;
use crate::describe::particles::ArchiveParticlesDescription;
use crate::describe::ppe::ArchivePpeDescription;
use crate::describe::shader_compiler::ArchiveShaderCompilerDescription;
use crate::describe::shaders::ArchiveShadersDescription;
use crate::describe::sound::ArchiveSoundEnvironmentDescription;
use crate::describe::spawn::{ArchiveLevelSpawnDescription, ArchiveSpawnDescription};
use crate::describe::thm::ArchiveThmDescription;

/// What the explorer can say about one entry it cannot draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveFormatDescription {
  Chunks {
    description: Box<ArchiveChunksDescription>,
  },
  Spawn {
    description: Box<ArchiveSpawnDescription>,
  },
  LevelSpawn {
    description: Box<ArchiveLevelSpawnDescription>,
  },
  Anm {
    description: Box<ArchiveAnmDescription>,
  },
  Detail {
    description: Box<ArchiveDetailModel>,
  },
  DetailLibrary {
    description: Box<ArchiveDetailLibraryDescription>,
  },
  LevelEnvMod {
    description: Box<ArchiveLevelEnvModDescription>,
  },
  LevelFogVol {
    description: Box<ArchiveLevelFogVolDescription>,
  },
  LevelGame {
    description: Box<ArchiveLevelGameDescription>,
  },
  GameMtl {
    description: Box<ArchiveGameMtlDescription>,
  },
  LevelHom {
    description: Box<ArchiveLevelHomDescription>,
  },
  LevelGeom {
    description: Box<ArchiveLevelGeomDescription>,
  },
  LightAnim {
    description: Box<ArchiveLightAnimDescription>,
  },
  ShaderCompiler {
    description: Box<ArchiveShaderCompilerDescription>,
  },
  SoundEnvironment {
    description: Box<ArchiveSoundEnvironmentDescription>,
  },
  LevelLights {
    description: Box<ArchiveLevelLightsDescription>,
  },
  LevelPsStatic {
    description: Box<ArchiveLevelPsStaticDescription>,
  },
  LevelSndStatic {
    description: Box<ArchiveLevelSndStaticDescription>,
  },
  LevelSom {
    description: Box<ArchiveLevelSomDescription>,
  },
  LevelWallmarks {
    description: Box<ArchiveLevelWallmarksDescription>,
  },
  Efd {
    description: Box<ArchiveEfdDescription>,
  },
  Level {
    description: Box<ArchiveLevelDescription>,
  },
  LevelAi {
    description: Box<ArchiveLevelAiDescription>,
  },
  LevelCollision {
    description: Box<ArchiveLevelCollisionDescription>,
  },
  Omf {
    description: Box<ArchiveOmfDescription>,
  },
  Particles {
    description: Box<ArchiveParticlesDescription>,
  },
  Ppe {
    description: Box<ArchivePpeDescription>,
  },
  Shaders {
    description: Box<ArchiveShadersDescription>,
  },
  Thm {
    description: Box<ArchiveThmDescription>,
  },
  Unsupported {
    reason: ArchiveDescribeRefusal,
  },
}

impl ArchiveFormatDescription {
  /// The description that says nothing was described, and why.
  pub(crate) fn refuse(reason: ArchiveDescribeRefusal) -> Self {
    Self::Unsupported { reason }
  }
}

/// Each describer's answer as the variant of the union it is, so a describer is named once beside its variant.
macro_rules! impl_format_description_from {
  ($($variant:ident => $description:ty),* $(,)?) => {
    $(
      impl From<$description> for ArchiveFormatDescription {
        fn from(description: $description) -> Self {
          Self::$variant {
            description: Box::new(description),
          }
        }
      }
    )*
  };
}

impl_format_description_from! {
  Chunks => ArchiveChunksDescription,
  Spawn => ArchiveSpawnDescription,
  LevelSpawn => ArchiveLevelSpawnDescription,
  Anm => ArchiveAnmDescription,
  Detail => ArchiveDetailModel,
  DetailLibrary => ArchiveDetailLibraryDescription,
  LevelEnvMod => ArchiveLevelEnvModDescription,
  LevelFogVol => ArchiveLevelFogVolDescription,
  LevelGame => ArchiveLevelGameDescription,
  GameMtl => ArchiveGameMtlDescription,
  LevelHom => ArchiveLevelHomDescription,
  LevelGeom => ArchiveLevelGeomDescription,
  LightAnim => ArchiveLightAnimDescription,
  ShaderCompiler => ArchiveShaderCompilerDescription,
  SoundEnvironment => ArchiveSoundEnvironmentDescription,
  LevelLights => ArchiveLevelLightsDescription,
  LevelPsStatic => ArchiveLevelPsStaticDescription,
  LevelSndStatic => ArchiveLevelSndStaticDescription,
  LevelSom => ArchiveLevelSomDescription,
  LevelWallmarks => ArchiveLevelWallmarksDescription,
  Efd => ArchiveEfdDescription,
  Level => ArchiveLevelDescription,
  LevelAi => ArchiveLevelAiDescription,
  LevelCollision => ArchiveLevelCollisionDescription,
  Omf => ArchiveOmfDescription,
  Particles => ArchiveParticlesDescription,
  Ppe => ArchivePpeDescription,
  Shaders => ArchiveShadersDescription,
  Thm => ArchiveThmDescription,
}
