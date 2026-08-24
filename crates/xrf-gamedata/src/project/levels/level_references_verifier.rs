use std::sync::Arc;

use xrf_db::{LevelShaderReference, LevelShadersChunk, ShaderLibraryFile};

use crate::GamedataFindingFactory;
use crate::project::levels::level_bundle::LevelBundle;
use crate::project::levels::level_engine_constants::LEVEL_FILE;
use crate::{Finding, GamedataProject, GamedataVerificationRule};

/// Findings and counters from resolving what a level's geometry references.
#[derive(Default)]
pub(crate) struct LevelReferencesOutcome {
  pub(crate) findings: Vec<Finding>,
  pub(crate) checked_count: u32,
  pub(crate) invalid_count: u32,
}

/// Resolves the shader and texture table the level geometry is built against.
pub(crate) struct LevelReferencesVerifier<'a> {
  bundle: &'a LevelBundle<'a>,
  shader_library: Option<&'a ShaderLibraryFile>,
}

impl<'a> LevelReferencesVerifier<'a> {
  pub(crate) fn new(bundle: &'a LevelBundle<'a>, shader_library: Option<&'a ShaderLibraryFile>) -> Self {
    Self { bundle, shader_library }
  }

  /// Load the shader library once per check.
  ///
  /// An absent or unreadable library is reported by the meshes check, so level shader names are
  /// simply left unresolved rather than reported twice.
  pub(crate) fn read_library(project: &GamedataProject) -> Option<Arc<ShaderLibraryFile>> {
    project
      .read_parsed(
        xrf_vfs::XrayAssetType::Shader,
        xrf_vfs::XrayAssetType::SHADER_LIBRARY_PATH,
        ShaderLibraryFile::read_from_chunk,
      )
      .ok()
  }

  pub(crate) fn verify(&self, shaders: &LevelShadersChunk) -> LevelReferencesOutcome {
    let mut outcome: LevelReferencesOutcome = LevelReferencesOutcome::default();
    let asset_path: String = self.bundle.file_path(LEVEL_FILE);

    for malformed in shaders.malformed() {
      outcome.findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsShaderReference,
        &asset_path,
        format!(
          "Level shader table entry [{malformed}] has no '/' delimiter, the renderer dereferences a null pointer on it"
        ),
      ));
    }

    for reference in shaders.references() {
      self.verify_reference(&asset_path, reference, &mut outcome);
    }

    outcome
  }

  fn verify_reference(&self, asset_path: &str, reference: &LevelShaderReference, outcome: &mut LevelReferencesOutcome) {
    outcome.checked_count += 1;

    if let Some(library) = self.shader_library
      && !library.contains_blender(&reference.shader)
    {
      outcome.invalid_count += 1;
      outcome.findings.push(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::LevelsShaderReference,
        asset_path,
        format!(
          "Level references shader [{}] that is not defined in shaders.xr",
          reference.shader
        ),
      ));
    }

    for texture in &reference.textures {
      outcome.checked_count += 1;

      if !self.bundle.resolves_texture(texture) {
        outcome.invalid_count += 1;
        outcome.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsTextureReference,
          asset_path,
          format!(
            "Level references missing texture [{texture}] through shader [{}]",
            reference.shader
          ),
        ));
      }
    }
  }
}
