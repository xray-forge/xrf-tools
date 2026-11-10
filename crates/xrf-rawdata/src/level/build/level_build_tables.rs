use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjMaterial, BuildPrjName, BuildPrjShader, BuildPrjTexture};

/// The material, shader and texture tables of a build, filled as the faces and glows name them
/// (`SceneBuilder::BuildMaterial`, `BuildShader`, `BuildShaderXRLC`, `BuildTexture`, `BuilderRemote.cpp:1208`): each
/// name kept once in the order first named, shaders compared exactly and textures without case, and a material once
/// per texture, shaders and sector.
#[derive(Default)]
pub(crate) struct LevelBuildTables {
  materials: Vec<BuildPrjMaterial>,
  render_shaders: Vec<String>,
  compiler_shaders: Vec<String>,
  textures: Vec<String>,
}

/// The tables as `build.prj` stores them.
pub(crate) struct LevelBuildTableRecords {
  pub(crate) materials: Vec<BuildPrjMaterial>,
  pub(crate) render_shaders: Vec<BuildPrjShader>,
  pub(crate) compiler_shaders: Vec<BuildPrjShader>,
  pub(crate) textures: Vec<BuildPrjTexture>,
}

impl LevelBuildTables {
  /// What a glow's material names for its compiler shader, which it has none of: `u16(-1)`.
  pub(crate) const NO_COMPILER_SHADER: u16 = u16::MAX;

  /// The material of `texture` under the render shader `shader` and the compiler shader `compiler_shader` (none for a
  /// glow) in `sector`, by its index.
  ///
  /// # Errors
  ///
  /// Returns an error when a name is empty, as the editor refuses one, or a table outgrows its `u16` indices.
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
      texture: Self::intern(&mut self.textures, texture, str::eq_ignore_ascii_case)?,
      shader: Self::intern(&mut self.render_shaders, shader, str::eq)?,
      compiler_shader: match compiler_shader {
        Some(name) => Self::intern(&mut self.compiler_shaders, name, str::eq)?,
        None => Self::NO_COMPILER_SHADER,
      },
      sector,
      reserved: 0,
      area_padding: 0,
      internal_max_area: 0,
    };

    match self.materials.iter().position(|held| {
      held.texture == material.texture
        && held.shader == material.shader
        && held.compiler_shader == material.compiler_shader
        && held.sector == material.sector
    }) {
      Some(index) => Self::get_index(index),
      None => {
        self.materials.push(material);
        Self::get_index(self.materials.len() - 1)
      }
    }
  }

  /// The tables as records, each name in a fresh `string128` buffer, and each texture's size, alpha and surface zero
  /// as the editor leaves them for the compiler to read.
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
      render_shaders: self.render_shaders.iter().map(shader).collect::<XrfResult<_>>()?,
      compiler_shaders: self.compiler_shaders.iter().map(shader).collect::<XrfResult<_>>()?,
      textures: self
        .textures
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

  /// The index of `name` in `names`, added last where `same` matches none.
  fn intern(names: &mut Vec<String>, name: &str, same: fn(&str, &str) -> bool) -> XrfResult<u16> {
    match names.iter().position(|held| same(held, name)) {
      Some(index) => Self::get_index(index),
      None => {
        names.push(name.to_owned());
        Self::get_index(names.len() - 1)
      }
    }
  }

  fn get_index(index: usize) -> XrfResult<u16> {
    u16::try_from(index)
      .ok()
      .filter(|index| *index != Self::NO_COMPILER_SHADER)
      .ok_or_else(|| XrfError::new_invalid_error("A build's table holds more entries than its u16 indices number"))
  }
}
