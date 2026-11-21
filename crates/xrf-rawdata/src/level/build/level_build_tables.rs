use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjMaterial, BuildPrjName, BuildPrjShader, BuildPrjTexture};

use crate::level::build::level_build_names::LevelBuildNames;
use crate::level::build::level_build_table_records::LevelBuildTableRecords;

/// The material, shader and texture tables of a build, filled as faces and glows name them
/// (`SceneBuilder::BuildMaterial`, `BuildShader`, `BuildShaderXRLC`, `BuildTexture`).
///
/// Each name is kept once in the order first named, shaders compared exactly and textures without case; a material
/// once per texture, shaders and sector. Every table is indexed by hash, as a large level names thousands.
pub(crate) struct LevelBuildTables {
  materials: Vec<BuildPrjMaterial>,
  /// Each material's index by its texture, shader, compiler shader and sector.
  material_indexes: HashMap<[u16; 4], u16>,
  render_shaders: LevelBuildNames,
  compiler_shaders: LevelBuildNames,
  textures: LevelBuildNames,
}

impl Default for LevelBuildTables {
  fn default() -> Self {
    Self {
      materials: Vec::new(),
      material_indexes: HashMap::new(),
      render_shaders: LevelBuildNames::exact(),
      compiler_shaders: LevelBuildNames::exact(),
      textures: LevelBuildNames::case_blind(),
    }
  }
}

impl LevelBuildTables {
  /// The compiler shader index of a glow's material, which has none: `u16(-1)`.
  pub(crate) const NO_COMPILER_SHADER: u16 = LevelBuildNames::NO_INDEX;

  /// The index of the material of `texture`, `shader`, `compiler_shader` (none for a glow) and `sector`.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture name is empty, as the editor refuses one, or a table outgrows its `u16` indices.
  pub(crate) fn get_material(
    &mut self,
    shader: &str,
    compiler_shader: Option<&str>,
    texture: &str,
    sector: u16,
  ) -> XrfResult<u16> {
    if texture.is_empty() {
      return Err(XrfError::new_invalid_error("Invalid texture name found"));
    }

    let material: BuildPrjMaterial = BuildPrjMaterial {
      texture: self.textures.intern(texture)?,
      shader: self.render_shaders.intern(shader)?,
      compiler_shader: match compiler_shader {
        Some(name) => self.compiler_shaders.intern(name)?,
        None => Self::NO_COMPILER_SHADER,
      },
      sector,
      reserved: 0,
      area_padding: 0,
      internal_max_area: 0,
    };
    let key: [u16; 4] = [
      material.texture,
      material.shader,
      material.compiler_shader,
      material.sector,
    ];

    if let Some(index) = self.material_indexes.get(&key) {
      return Ok(*index);
    }

    let index: u16 = LevelBuildNames::get_index(self.materials.len())?;

    self.materials.push(material);
    self.material_indexes.insert(key, index);

    Ok(index)
  }

  /// The tables as records, each name in a fresh `string128`, each texture's size, alpha and surface left zero for the
  /// compiler to read, as the editor leaves them.
  ///
  /// # Errors
  ///
  /// Returns an error when a name is not Windows-1251 or overfills its buffer.
  pub(crate) fn into_records(self) -> XrfResult<LevelBuildTableRecords> {
    let shader = |name: &String| -> XrfResult<BuildPrjShader> {
      Ok(BuildPrjShader {
        name: BuildPrjName::new(name, BuildPrjName::LONG_SIZE)?,
      })
    };

    Ok(LevelBuildTableRecords {
      materials: self.materials,
      render_shaders: self
        .render_shaders
        .get_names()
        .iter()
        .map(shader)
        .collect::<XrfResult<_>>()?,
      compiler_shaders: self
        .compiler_shaders
        .get_names()
        .iter()
        .map(shader)
        .collect::<XrfResult<_>>()?,
      textures: self
        .textures
        .get_names()
        .iter()
        .map(|name| {
          Ok(BuildPrjTexture {
            name: BuildPrjName::new(name, BuildPrjName::LONG_SIZE)?,
            width: 0,
            height: 0,
            has_alpha: 0,
            surface: 0,
          })
        })
        .collect::<XrfResult<_>>()?,
    })
  }
}
