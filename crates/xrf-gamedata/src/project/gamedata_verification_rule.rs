use derive_more::Display;

/// Stable rule identifiers assigned to gamedata verification findings.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum GamedataVerificationRule {
  #[display("animations.hud-item")]
  AnimationsHudItem,
  #[display("animations.motion-collision")]
  AnimationsMotionCollision,
  /// An object motion (`.anm`) that does not read.
  #[display("animations.object-motion-read")]
  AnimationsObjectMotionRead,
  /// An object motion that does not write back to the bytes it was read from.
  #[display("animations.object-motion-round-trip")]
  AnimationsObjectMotionRoundTrip,
  /// An object motion channel that evaluates to something other than a finite number.
  #[display("animations.object-motion-value")]
  AnimationsObjectMotionValue,
  #[display("animations.player-hud")]
  AnimationsPlayerHud,
  /// A post-process effect whose noise granularity reaches zero or below while it plays, which the engine stops on.
  #[display("animations.post-process-noise-granularity")]
  AnimationsPostProcessNoiseGranularity,
  /// A post-process effect (`.ppe`) that does not read.
  #[display("animations.post-process-read")]
  AnimationsPostProcessRead,
  /// A post-process effect that does not write back to the bytes it was read from.
  #[display("animations.post-process-round-trip")]
  AnimationsPostProcessRoundTrip,
  /// A post-process effect channel that evaluates to something other than a finite number.
  #[display("animations.post-process-value")]
  AnimationsPostProcessValue,
  #[display("checks.execution")]
  CheckExecution,
  #[display("collisions.unreachable")]
  CollisionsUnreachable,
  #[display("coverage.skipped-mount")]
  CoverageSkippedMount,
  /// A texture an environment config names that the game does not hold.
  #[display("environment.asset")]
  EnvironmentAsset,
  /// Loaded by the engine, against a convention strong enough to be a mistake.
  #[display("environment.convention")]
  EnvironmentConvention,
  /// Refused or misread by the engine.
  #[display("environment.engine")]
  EnvironmentEngine,
  /// A section naming another the engine cannot find.
  #[display("environment.reference")]
  EnvironmentReference,
  #[display("levels.ai-guid")]
  LevelsAiGuid,
  #[display("levels.ai-node-count")]
  LevelsAiNodeCount,
  #[display("levels.ai-version")]
  LevelsAiVersion,
  #[display("levels.cform-version")]
  LevelsCformVersion,
  #[display("levels.details-pair")]
  LevelsDetailsPair,
  #[display("levels.file-empty")]
  LevelsFileEmpty,
  #[display("levels.file-truncated")]
  LevelsFileTruncated,
  /// A collision face made of a material the library does not define, or of a dynamic one, which stops level loading.
  #[display("levels.game-material")]
  LevelsGameMaterial,
  #[display("levels.graph-duplicate")]
  LevelsGraphDuplicate,
  #[display("levels.graph-guid")]
  LevelsGraphGuid,
  #[display("levels.header-version")]
  LevelsHeaderVersion,
  #[display("levels.level-guid")]
  LevelsLevelGuid,
  #[display("levels.ltx-read")]
  LevelsLtxRead,
  #[display("levels.map-texture")]
  LevelsMapTexture,
  #[display("levels.missing-bundle")]
  LevelsMissingBundle,
  #[display("levels.missing-file")]
  LevelsMissingFile,
  #[display("levels.orphan-bundle")]
  LevelsOrphanBundle,
  #[display("levels.roster-conflict")]
  LevelsRosterConflict,
  #[display("levels.shader-reference")]
  LevelsShaderReference,
  #[display("levels.shaders-chunk")]
  LevelsShadersChunk,
  #[display("levels.texture-reference")]
  LevelsTextureReference,
  #[display("levels.undeclared-map")]
  LevelsUndeclaredMap,
  /// A statement the LTX dialect reports the engine loads and then ignores.
  #[display("ltx.dialect")]
  LtxDialect,
  #[display("ltx.formatting")]
  LtxFormatting,
  #[display("ltx.schema")]
  LtxSchema,
  #[display("ltx.verification")]
  LtxVerification,
  /// A material the engine looks up by a name of its own that the library does not define as the engine needs it.
  #[display("materials.engine")]
  MaterialsEngine,
  /// The game material library is missing or does not read.
  #[display("materials.library")]
  MaterialsLibrary,
  /// A pair list holding more entries than the engine's load asserts it may.
  #[display("materials.list")]
  MaterialsList,
  /// A wallmark texture a pair names that the game does not hold.
  #[display("materials.mark")]
  MaterialsMark,
  /// A pair naming a material number the library does not define.
  #[display("materials.pair")]
  MaterialsPair,
  /// A config or spawn object naming a material the library does not define.
  #[display("materials.reference")]
  MaterialsReference,
  #[display("meshes.chunk-residue")]
  MeshesChunkResidue,
  /// A bone made of a material the library does not define, or of one that is not dynamic.
  #[display("meshes.game-material")]
  MeshesGameMaterial,
  #[display("meshes.motion-label")]
  MeshesMotionLabel,
  #[display("meshes.motion-read")]
  MeshesMotionRead,
  #[display("meshes.motion-validation")]
  MeshesMotionValidation,
  #[display("meshes.read")]
  MeshesRead,
  #[display("meshes.shader-library")]
  MeshesShaderLibrary,
  #[display("meshes.validation")]
  MeshesValidation,
  #[display("particles.library")]
  ParticlesLibrary,
  #[display("particles.texture")]
  ParticlesTexture,
  #[display("particles-usage.reference")]
  ParticlesUsageReference,
  #[display("particles-usage.spawn")]
  ParticlesUsageSpawn,
  #[display("particles-usage.spawn-custom-data")]
  ParticlesUsageSpawnCustomData,
  /// A runtime script that does not read as Windows-1251 text.
  #[display("scripts.read")]
  ScriptsRead,
  /// A runtime script LuaJIT's parser rejects.
  #[display("scripts.syntax")]
  ScriptsSyntax,
  #[display("shaders.include-cycle")]
  ShadersIncludeCycle,
  #[display("shaders.include-missing")]
  ShadersIncludeMissing,
  #[display("shaders.include-syntax")]
  ShadersIncludeSyntax,
  #[display("shaders.lua-syntax")]
  ShadersLuaSyntax,
  #[display("shaders.renderer-root")]
  ShadersRendererRoot,
  #[display("shaders.source-invalid")]
  ShadersSourceInvalid,
  #[display("shaders.source-read")]
  ShadersSourceRead,
  #[display("sounds.files")]
  SoundsFiles,
  #[display("sounds.references")]
  SoundsReferences,
  #[display("spawns.read")]
  SpawnsRead,
  #[display("textures.bump")]
  TexturesBump,
  #[display("textures.bump-companion")]
  TexturesBumpCompanion,
  #[display("textures.bump-declaration")]
  TexturesBumpDeclaration,
  #[display("textures.chunk-residue")]
  TexturesChunkResidue,
  #[display("textures.read")]
  TexturesRead,
  #[display("textures.dds")]
  TexturesValidation,
  #[display("weapons.validation")]
  WeaponsValidation,
}

#[cfg(test)]
mod tests {
  use super::GamedataVerificationRule;

  #[test]
  fn displays_stable_rule_ids() {
    assert_eq!(
      GamedataVerificationRule::ParticlesUsageSpawnCustomData.to_string(),
      "particles-usage.spawn-custom-data"
    );
    assert_eq!(
      GamedataVerificationRule::ShadersIncludeMissing.to_string(),
      "shaders.include-missing"
    );
  }
}
