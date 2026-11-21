use std::path::PathBuf;

use xrf_dds::{DdsFile, DdsMetadata};
use xrf_error::{XrfError, XrfResult};
use xrf_level::BuildPrjFile;
use xrf_thm::{ThmFile, ThmTextureFlag, ThmTextureParamChunk};

use crate::input::compiler_textures::CompilerTextures;

/// One `build.prj` texture as `CBuild::Load` loads it.
///
/// The size and flags come from its `.thm`; the image is loaded where the texture has alpha or is lit implicitly and is
/// not a `lod\` texture, and its size then wins over the `.thm`'s. The merged impostors' atlas, `level_lods`, has no
/// `.thm` read: 1024 a side, alpha, no image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompilerTexture {
  /// The name cut at its first dot and lowered (`CompilerTextures::get_name`), as visuals name it.
  pub name: String,
  pub width: u32,
  pub height: u32,
  pub has_alpha: bool,
  /// Whether xrLC loaded its image (`THM.HasSurface`).
  pub(crate) has_image: bool,
  pub(crate) is_implicitly_lit: bool,
}

impl CompilerTexture {
  /// The atlas's side, which xrLC sets for it.
  pub(crate) const UNLOADED_SIDE: u32 = 1024;

  /// The folder of textures whose images xrLC never loads.
  const LOD_FOLDER: &'static str = "lod\\";

  /// Every texture `project` names, found under the first of `roots` holding it.
  ///
  /// # Errors
  ///
  /// Returns an error listing the textures with no texture `.thm`, or for an image xrLC loads that cannot be read.
  pub(crate) fn resolve_all(project: &BuildPrjFile, roots: &[PathBuf]) -> XrfResult<Vec<Self>> {
    let mut missing: Vec<String> = Vec::new();
    let mut textures: Vec<Self> = Vec::with_capacity(project.textures.len());

    for texture in &project.textures {
      let name: String = CompilerTextures::get_name(&texture.name.value);

      if name == CompilerTextures::UNLOADED {
        textures.push(Self {
          name,
          width: Self::UNLOADED_SIDE,
          height: Self::UNLOADED_SIDE,
          has_alpha: true,
          has_image: false,
          is_implicitly_lit: false,
        });
        continue;
      }

      let parameters: Option<ThmTextureParamChunk> = CompilerTextures::find(roots, &name, "thm")?
        .map(|path| ThmFile::read_from_path(&path))
        .transpose()?
        .and_then(|thumbnail| thumbnail.texture_param);
      let Some(parameters) = parameters else {
        missing.push(name);
        continue;
      };
      let has_alpha: bool = parameters.format.has_alpha_channel();
      let is_implicitly_lit: bool = parameters.flags.has(ThmTextureFlag::ImplicitLighted);
      let has_image: bool = (has_alpha || is_implicitly_lit) && !name.starts_with(Self::LOD_FOLDER);
      let [width, height] = if has_image {
        let image: PathBuf = CompilerTextures::find(roots, &name, "dds")?.ok_or_else(|| {
          XrfError::new_not_found_error(format!(
            "Texture {name}, whose image xrLC loads, has no image under the texture roots"
          ))
        })?;
        let metadata: DdsMetadata = DdsFile::read_metadata_from_path(&image)?;

        [metadata.width, metadata.height]
      } else {
        [parameters.width, parameters.height]
      };

      textures.push(Self {
        name,
        width,
        height,
        has_alpha,
        has_image,
        is_implicitly_lit,
      });
    }

    match missing.as_slice() {
      [] => Ok(textures),
      [first, ..] => Err(XrfError::new_not_found_error(format!(
        "{} of the textures build.prj names have no .thm describing a texture under the texture roots, such as {first}",
        missing.len()
      ))),
    }
  }

  /// Whether faces of it are opaque to xrLC (`CacheOpacity`): no alpha, or alpha with no image loaded.
  pub(crate) fn is_opaque(&self) -> bool {
    !self.has_alpha || !self.has_image
  }
}
