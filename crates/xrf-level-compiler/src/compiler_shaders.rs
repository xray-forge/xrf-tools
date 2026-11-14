use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjFace, BuildPrjFile, BuildPrjMuModel};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

/// Each `build.prj` material's compiler shader, as the SDK's compilers resolve it (`post_process_materials`,
/// `utils/Shader_xrLC.h:130`): the project's compiler shader the material names, found by name in the library, `None`
/// for a material with none.
#[derive(Clone, Debug)]
pub struct CompilerShaders {
  by_material: Vec<Option<ShaderCompilerShader>>,
}

impl CompilerShaders {
  /// A material's compiler shader where it has none, as a glow's material has.
  pub const NONE: u16 = u16::MAX;

  /// Every material of `project` resolved in `library`.
  ///
  /// # Errors
  ///
  /// Returns an error naming a material whose compiler shader the project does not hold, or every compiler shader the
  /// library lacks, on which the compilers assert.
  pub fn resolve(project: &BuildPrjFile, library: &ShaderCompilerFile) -> XrfResult<Self> {
    let mut missing: Vec<&str> = Vec::new();
    let mut by_material: Vec<Option<ShaderCompilerShader>> = Vec::with_capacity(project.materials.len());

    for (index, material) in project.materials.iter().enumerate() {
      if material.compiler_shader == Self::NONE {
        by_material.push(None);
        continue;
      }

      let Some(name) = project.compiler_shaders.get(usize::from(material.compiler_shader)) else {
        return Err(XrfError::new_invalid_error(format!(
          "build.prj material {index} names compiler shader {} of {}, which it does not hold",
          material.compiler_shader,
          project.compiler_shaders.len()
        )));
      };

      match library.find_shader(&name.name.value) {
        Some(shader) => by_material.push(Some(shader.clone())),
        None => {
          missing.push(&name.name.value);
          by_material.push(None);
        }
      }
    }

    missing.sort_unstable();
    missing.dedup();

    match missing.as_slice() {
      [] => Ok(Self { by_material }),
      names => Err(XrfError::new_not_found_error(format!(
        "build.prj names compiler shaders the library lacks: {}",
        names.join(", ")
      ))),
    }
  }

  /// The compiler shader of material `material`, `None` for one with none or one the project does not hold.
  pub fn get(&self, material: u16) -> Option<&ShaderCompilerShader> {
    self.by_material.get(usize::from(material)).and_then(Option::as_ref)
  }

  /// The compiler shader of face `number` of multi-use model `model`, which xrLC reads without a check.
  ///
  /// # Errors
  ///
  /// Returns an error for a face of a material with no compiler shader or one the project does not hold.
  pub(crate) fn require(
    &self,
    model: &BuildPrjMuModel,
    number: usize,
    face: &BuildPrjFace,
  ) -> XrfResult<&ShaderCompilerShader> {
    self.get(face.material).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "Multi-use model {} face {number} is of material {}, which has no compiler shader",
        model.name, face.material
      ))
    })
  }
}
