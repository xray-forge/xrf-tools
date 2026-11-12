use std::path::PathBuf;

use rayon::prelude::*;
use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsFile, GrayImage};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::TraceHit;
use xrf_level::BuildPrjFile;
use xrf_math::Vector2d;
use xrf_shaders::ShaderCompilerShader;
use xrf_thm::{ThmFile, ThmTextureParamChunk};
use xrf_utils::format_path;

use crate::compiler_textures::CompilerTextures;
use crate::shadow::shadow_class::ShadowClass;
use crate::shadow::shadow_class_counts::ShadowClassCounts;
use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;
use crate::shadow::shadow_texture::ShadowTexture;

/// How each ray model face meets a light ray, as the SDK's lighters read it from `build.prj`, `shaders_xrlc.xr` and the
/// textures (`xrLoad`, `xrLC_Light/global_calculation_data.cpp:34`; `getLastRP_Scale`, `detail_slot_calculate.cpp:93`):
///
/// - a face whose material's compiler shader casts no shadow passes the ray;
/// - a face whose texture has an alpha channel, by its `.thm`, is dimmed through that alpha, unless the texture is a
///   `lod\` one or the merged impostors' atlas, whose image the compilers never load;
/// - every other face blocks it.
///
/// Faces are known by their index into the ray model, the tracer's face index.
#[derive(Clone, Debug)]
pub struct ShadowMaterials {
  /// One per ray model face.
  faces: Vec<ShadowFace>,
  /// The texture and texture coordinates of each face dimming through alpha.
  alpha_faces: Vec<ShadowAlphaFace>,
  /// The textures alpha faces name, each once.
  textures: Vec<ShadowTexture>,
}

/// One ray model face's part in a light ray.
#[derive(Clone, Copy, Debug)]
enum ShadowFace {
  Passes,
  Opaque,
  /// By its index among the alpha faces.
  Alpha(u32),
}

#[derive(Clone, Copy, Debug)]
struct ShadowAlphaFace {
  /// By its index among the loaded textures.
  texture: u32,
  coordinates: [Vector2d; 3],
}

/// One `build.prj` material's part in a light ray.
#[derive(Clone, Copy, Debug)]
enum ShadowMaterial {
  Passes,
  Opaque,
  Alpha(ShadowImageSource),
}

/// An alpha texture to load: its index in `build.prj`, and its size by its `.thm`.
#[derive(Clone, Copy, Debug)]
struct ShadowImageSource {
  texture: usize,
  size: [u32; 2],
}

impl ShadowMaterials {
  /// The folder whose textures' images the compilers never load (`global_calculation_data.cpp:168`).
  const LOD_FOLDER: &'static str = "lod\\";

  /// What `build.prj` leaves in a material with no compiler shader.
  const NO_COMPILER_SHADER: u16 = u16::MAX;

  /// Reads a level's shadow materials: every texture's `.thm`, and the image of each alpha texture a face casting
  /// shadows names, decoded as the compilers' image library decodes it.
  ///
  /// # Errors
  ///
  /// Returns an error where the SDK's compilers stop: a texture with no `.thm` or none describing a texture under the
  /// roots, a compiler shader the library lacks, a face naming a material `build.prj` does not hold or one with no
  /// compiler shader, an alpha image missing, unreadable, or of another size than its `.thm` gives.
  pub fn load(request: &ShadowMaterialsRequest<'_>) -> XrfResult<Self> {
    let names: Vec<String> = request
      .project
      .textures
      .iter()
      .map(|texture| CompilerTextures::get_name(&texture.name.value))
      .collect();
    let alpha_sizes: Vec<Option<[u32; 2]>> = Self::read_alpha_sizes(&names, request.textures)?;
    let materials: Vec<Option<ShadowMaterial>> = Self::classify_materials(request, &alpha_sizes)?;
    let mut faces: Vec<ShadowFace> = Vec::with_capacity(request.faces.len());
    let mut alpha_faces: Vec<ShadowAlphaFace> = Vec::new();
    // Each project texture's index among the loaded textures, and the textures loaded, in order.
    let mut texture_slots: Vec<Option<u32>> = vec![None; names.len()];
    let mut loaded: Vec<ShadowImageSource> = Vec::new();

    for (index, face) in request.faces.iter().enumerate() {
      let material: ShadowMaterial = materials
        .get(usize::from(face.material))
        .copied()
        .flatten()
        .ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Ray model face {index} names material {}, which build.prj does not hold or gives no compiler shader",
            face.material
          ))
        })?;

      faces.push(match material {
        ShadowMaterial::Passes => ShadowFace::Passes,
        ShadowMaterial::Opaque => ShadowFace::Opaque,
        ShadowMaterial::Alpha(source) => {
          let slot: u32 = *texture_slots[source.texture].get_or_insert_with(|| {
            loaded.push(source);

            (loaded.len() - 1) as u32
          });

          alpha_faces.push(ShadowAlphaFace {
            texture: slot,
            coordinates: face.texture_coordinates,
          });

          ShadowFace::Alpha((alpha_faces.len() - 1) as u32)
        }
      });
    }

    let textures: Vec<ShadowTexture> = loaded
      .par_iter()
      .map(|source| Self::read_texture(&names[source.texture], source.size, request.textures))
      .collect::<XrfResult<_>>()?;

    Ok(Self {
      faces,
      alpha_faces,
      textures,
    })
  }

  /// How a ray model face meets a light ray.
  ///
  /// # Panics
  ///
  /// Panics when `face` is not a face of the ray model these were loaded for.
  pub fn get_class(&self, face: u32) -> ShadowClass {
    match self.faces[face as usize] {
      ShadowFace::Passes => ShadowClass::Passes,
      ShadowFace::Opaque => ShadowClass::Opaque,
      ShadowFace::Alpha(_) => ShadowClass::Alpha,
    }
  }

  /// How much of a light ray passes the face it hits: 1 for a face passing it, 0 for one blocking it, and `1 - alpha`
  /// at the hit's texture coordinates for an alpha one, weighted by its barycentrics as the SDK weighs them.
  ///
  /// # Panics
  ///
  /// Panics when the hit's face is not a face of the ray model these were loaded for.
  pub fn get_transmittance(&self, hit: &TraceHit) -> f32 {
    match self.faces[hit.face as usize] {
      ShadowFace::Passes => 1.0,
      ShadowFace::Opaque => 0.0,
      ShadowFace::Alpha(index) => {
        let face: &ShadowAlphaFace = &self.alpha_faces[index as usize];
        let [first, second, third] = &face.coordinates;
        let weight: f32 = 1.0 - hit.u - hit.v;

        self.textures[face.texture as usize].get_transmittance(Vector2d {
          x: first.x * weight + second.x * hit.u + third.x * hit.v,
          y: first.y * weight + second.y * hit.u + third.y * hit.v,
        })
      }
    }
  }

  /// How many ray model faces are of each class.
  pub fn count_classes(&self) -> ShadowClassCounts {
    self
      .faces
      .iter()
      .fold(ShadowClassCounts::default(), |mut counts, face| {
        match face {
          ShadowFace::Passes => counts.passes += 1,
          ShadowFace::Opaque => counts.opaque += 1,
          ShadowFace::Alpha(_) => counts.alpha += 1,
        }

        counts
      })
  }

  /// The textures alpha faces dim light through, in the order faces first name them.
  pub fn get_textures(&self) -> &[ShadowTexture] {
    &self.textures
  }

  /// Each texture's size where the compilers load its image for its alpha, by its `.thm`: an alpha channel, and
  /// neither the merged impostors' atlas, which they take as having alpha but no image, nor a `lod\` texture.
  fn read_alpha_sizes(names: &[String], roots: &[PathBuf]) -> XrfResult<Vec<Option<[u32; 2]>>> {
    let mut missing: Vec<&str> = Vec::new();
    let mut sizes: Vec<Option<[u32; 2]>> = Vec::with_capacity(names.len());

    for name in names {
      if name == CompilerTextures::UNLOADED {
        sizes.push(None);
        continue;
      }

      let parameters: Option<ThmTextureParamChunk> = CompilerTextures::find(roots, name, "thm")
        .map(|path| ThmFile::read_from_path::<XRayByteOrder, _>(&path))
        .transpose()?
        .and_then(|thumbnail| thumbnail.texture_param);
      let Some(parameters) = parameters else {
        missing.push(name);
        sizes.push(None);
        continue;
      };

      sizes.push(
        (parameters.format.has_alpha_channel() && !name.starts_with(Self::LOD_FOLDER))
          .then_some([parameters.width, parameters.height]),
      );
    }

    match missing.as_slice() {
      [] => Ok(sizes),
      [first, ..] => Err(XrfError::new_not_found_error(format!(
        "{} of the textures build.prj names have no .thm describing a texture under the texture roots, such as {first}",
        missing.len()
      ))),
    }
  }

  /// Each `build.prj` material's part in a light ray, `None` for one with no compiler shader: its compiler shader's
  /// shadow flag by its name in the library (`post_process_materials`, `Shader_xrLC.h:130`), then its texture's alpha.
  fn classify_materials(
    request: &ShadowMaterialsRequest<'_>,
    alpha_sizes: &[Option<[u32; 2]>],
  ) -> XrfResult<Vec<Option<ShadowMaterial>>> {
    let project: &BuildPrjFile = request.project;
    let casts_shadow: Vec<Option<bool>> = project
      .compiler_shaders
      .iter()
      .map(|shader| {
        request
          .library
          .find_shader(&shader.name.value)
          .map(ShaderCompilerShader::casts_shadow)
      })
      .collect();
    let mut missing: Vec<&str> = Vec::new();
    let mut materials: Vec<Option<ShadowMaterial>> = Vec::with_capacity(project.materials.len());

    for (index, material) in project.materials.iter().enumerate() {
      if material.compiler_shader == Self::NO_COMPILER_SHADER {
        materials.push(None);
        continue;
      }

      let shader: usize = usize::from(material.compiler_shader);
      let texture: usize = usize::from(material.texture);
      let (Some(casts_shadow), Some(alpha_size)) = (casts_shadow.get(shader), alpha_sizes.get(texture)) else {
        return Err(XrfError::new_invalid_error(format!(
          "build.prj material {index} names compiler shader {shader} and texture {texture}, which it does not hold"
        )));
      };

      materials.push(match casts_shadow {
        None => {
          missing.push(&project.compiler_shaders[shader].name.value);
          None
        }
        Some(false) => Some(ShadowMaterial::Passes),
        Some(true) => Some(match *alpha_size {
          Some(size) => ShadowMaterial::Alpha(ShadowImageSource { texture, size }),
          None => ShadowMaterial::Opaque,
        }),
      });
    }

    missing.sort_unstable();
    missing.dedup();

    match missing.as_slice() {
      [] => Ok(materials),
      names => Err(XrfError::new_not_found_error(format!(
        "build.prj names compiler shaders the library lacks: {}",
        names.join(", ")
      ))),
    }
  }

  /// An alpha texture's image, its alpha decoded as the compilers' image library decodes it.
  fn read_texture(name: &str, size: [u32; 2], roots: &[PathBuf]) -> XrfResult<ShadowTexture> {
    let path: PathBuf = CompilerTextures::find(roots, name, "dds").ok_or_else(|| {
      XrfError::new_not_found_error(format!("The alpha texture {name} has no .dds under the texture roots"))
    })?;
    let alpha: GrayImage = DdsFile::read_from_path(&path)?.decode_sdk_alpha()?;

    if alpha.dimensions() != (size[0], size[1]) {
      return Err(XrfError::new_invalid_error(format!(
        "{} is {} by {}, and its .thm says {} by {}",
        format_path(&path),
        alpha.width(),
        alpha.height(),
        size[0],
        size[1]
      )));
    }

    Ok(ShadowTexture {
      name: name.to_owned(),
      alpha,
    })
  }
}
