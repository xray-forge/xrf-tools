//! How each ray model face meets a light ray, from the project, the compiler shader library and the textures.

use std::path::{Path, PathBuf};

use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_error::XrfResult;
use xrf_geometry::TraceHit;
use xrf_level::{
  BuildCformRaycastFace, BuildPrjFile, BuildPrjMaterial, BuildPrjName, BuildPrjParams, BuildPrjShader, BuildPrjTexture,
};
use xrf_math::Vector2d;
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmFile, ThmFormat, ThmTextureParamChunk};

use crate::{ShadowClass, ShadowClassCounts, ShadowMaterials, ShadowMaterialsRequest};

/// The compiler shader casting shadows, and the one letting light through.
const CASTING: u16 = 0;
const CLEAR: u16 = 1;

/// The textures `build.prj` names, by index.
const WALL: u16 = 0;
const LEAF: u16 = 1;
const LOD_LEAF: u16 = 2;
const LEVEL_LODS: u16 = 3;

/// What a project, a library and two texture roots hold.
struct Scene {
  project: BuildPrjFile,
  library: ShaderCompilerFile,
  roots: Vec<PathBuf>,
}

impl Scene {
  /// A wall, a leaf with alpha, a `lod\` leaf and the impostors' atlas, under materials pairing them with the two
  /// compiler shaders, the leaf's image in the second root and every `.thm` in the first.
  fn new(name: &str) -> XrfResult<Self> {
    let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;
    let roots: Vec<PathBuf> = vec![root.join("first"), root.join("second")];

    Self::write_thm(&roots[0], "wall", ThmFormat::Dxt1)?;
    Self::write_thm(&roots[0], "trees\\leaf", ThmFormat::Dxt5)?;
    Self::write_thm(&roots[0], "lod\\lod_leaf", ThmFormat::Dxt5)?;
    Self::write_leaf(&roots[1].join("trees").join("leaf.dds"), 8)?;

    let materials: Vec<BuildPrjMaterial> = [
      (WALL, CASTING),
      (LEAF, CASTING),
      (LEAF, CLEAR),
      (LOD_LEAF, CASTING),
      (LEVEL_LODS, CASTING),
      (WALL, u16::MAX),
    ]
    .into_iter()
    .map(|(texture, compiler_shader)| BuildPrjMaterial {
      texture,
      shader: 0,
      compiler_shader,
      sector: 0,
      reserved: 0,
      area_padding: 0,
      internal_max_area: 0,
    })
    .collect();

    Ok(Self {
      project: BuildPrjFile {
        version: BuildPrjFile::CURRENT_VERSION,
        params: BuildPrjParams {
          smooth_angle: 0.0,
          weld_distance: 0.0,
          lightmap_pixels_per_meter: 0.0,
          lightmap_jitter_samples: 0,
          lightmap_rms_zero: 0,
          lightmap_rms: 0,
          quality: 0,
          quality_reserved: 0,
          reserved: [0.0; 6],
        },
        vertices: Vec::new(),
        faces: Vec::new(),
        smoothing_groups: None,
        materials,
        render_shaders: Vec::new(),
        compiler_shaders: Self::shaders(&["default", "def_shaders\\def_trans"])?,
        textures: ["Wall", "trees\\Leaf.dds", "lod\\lod_leaf", "level_lods"]
          .into_iter()
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
        glows: Vec::new(),
        portals: Vec::new(),
        light_controls: Vec::new(),
        static_lights: Vec::new(),
        dynamic_lights: Vec::new(),
        lods: Vec::new(),
        mu_models: Vec::new(),
        mu_references: Vec::new(),
      },
      library: ShaderCompilerFile {
        shaders: vec![
          Self::library_shader("DEFAULT", ShaderCompilerShader::CASTS_SHADOW),
          Self::library_shader("def_shaders\\def_trans", 0),
        ],
      },
      roots,
    })
  }

  /// One face a material, the alpha leaf's corners at texture coordinates whose second sits in its opaque half.
  fn faces(materials: &[u16]) -> Vec<BuildCformRaycastFace> {
    materials
      .iter()
      .map(|material| BuildCformRaycastFace {
        material: *material,
        material_padding: 0,
        game_material: 0,
        texture_coordinates: [
          Vector2d { x: 0.0, y: 0.0 },
          Vector2d { x: 0.75, y: 0.0 },
          Vector2d { x: 0.0, y: 0.5 },
        ],
        reserved: 0,
        trailing_padding: 0,
      })
      .collect()
  }

  fn load(&self, faces: &[BuildCformRaycastFace]) -> XrfResult<ShadowMaterials> {
    ShadowMaterials::load(&ShadowMaterialsRequest {
      project: &self.project,
      faces,
      library: &self.library,
      textures: &self.roots,
    })
  }

  fn shaders(names: &[&str]) -> XrfResult<Vec<BuildPrjShader>> {
    names
      .iter()
      .map(|name| {
        Ok(BuildPrjShader {
          name: BuildPrjName::new(name, BuildPrjName::LONG_SIZE)?,
        })
      })
      .collect()
  }

  fn library_shader(name: &str, flags: u32) -> ShaderCompilerShader {
    ShaderCompilerShader {
      name: String::from(name),
      name_trailing: vec![0],
      flags,
      vertex_translucency: 0.5,
      vertex_ambient: 0.0,
      lightmap_density: 1.0,
    }
  }

  /// A `.thm` describing an 8 by 4 texture of `format`.
  fn write_thm(root: &Path, name: &str, format: ThmFormat) -> XrfResult {
    let mut thumbnail: ThmFile = ThmFile::new_texture();
    let path: PathBuf = name.split('\\').fold(root.to_path_buf(), |path, part| path.join(part));

    thumbnail.texture_param = Some(ThmTextureParamChunk {
      format,
      width: 8,
      height: 4,
      ..ThmTextureParamChunk::default()
    });
    std::fs::create_dir_all(path.parent().unwrap_or(root))?;
    thumbnail.write_to_path::<XRayByteOrder, _>(&path.with_extension("thm"))
  }

  /// A DXT5 leaf `width` by 4, clear in its left block and opaque in the rest.
  fn write_leaf(path: &Path, width: u32) -> XrfResult {
    let image: RgbaImage = RgbaImage::from_fn(width, 4, |x, _| Rgba([0, 128, 0, if x < 4 { 0 } else { 255 }]));

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast)
      .encode(&DdsMipChain::build(&image, DdsMipmaps::Disabled)?)?
      .write_to_path(path)
  }
}

fn hit(face: u32, u: f32, v: f32) -> TraceHit {
  TraceHit {
    face,
    distance: 1.0,
    u,
    v,
  }
}

#[test]
fn classes_each_face_by_its_compiler_shader_and_texture() -> XrfResult {
  let scene: Scene = Scene::new("classes")?;
  let materials: ShadowMaterials = scene.load(&Scene::faces(&[0, 1, 2, 3, 4, 1]))?;

  assert_eq!(
    (0..6).map(|face| materials.get_class(face)).collect::<Vec<_>>(),
    [
      ShadowClass::Opaque,
      ShadowClass::Alpha,
      ShadowClass::Passes,
      ShadowClass::Opaque,
      ShadowClass::Opaque,
      ShadowClass::Alpha,
    ]
  );
  assert_eq!(
    materials.count_classes(),
    ShadowClassCounts {
      passes: 1,
      alpha: 2,
      opaque: 3,
    }
  );
  assert_eq!(
    materials
      .get_textures()
      .iter()
      .map(|texture| texture.name.as_str())
      .collect::<Vec<_>>(),
    ["trees\\leaf"]
  );

  Ok(())
}

#[test]
fn passes_blocks_or_dims_the_light_where_a_ray_crosses_a_face() -> XrfResult {
  let scene: Scene = Scene::new("transmittance")?;
  let materials: ShadowMaterials = scene.load(&Scene::faces(&[0, 1, 2]))?;

  assert_eq!(materials.get_transmittance(&hit(0, 0.0, 0.0)), 0.0);
  assert_eq!(materials.get_transmittance(&hit(2, 0.5, 0.5)), 1.0);
  // The leaf's first corner sits in its clear block, its second in its opaque one.
  assert_eq!(materials.get_transmittance(&hit(1, 0.0, 0.0)), 1.0);
  assert_eq!(materials.get_transmittance(&hit(1, 1.0, 0.0)), 0.0);

  Ok(())
}

#[test]
fn refuses_what_stops_the_sdk_s_lighter() -> XrfResult {
  let scene: Scene = Scene::new("refusals")?;
  let error = |result: XrfResult<ShadowMaterials>| result.map(|_| ()).unwrap_err().to_string();

  // A face of a material with no compiler shader, or one the project does not hold.
  assert!(error(scene.load(&Scene::faces(&[5]))).contains("material 5"));
  assert!(error(scene.load(&Scene::faces(&[9]))).contains("material 9"));

  // A compiler shader the library lacks.
  let mut lacking: Scene = Scene::new("refusals_library")?;

  lacking.library.shaders.pop();
  assert!(error(lacking.load(&Scene::faces(&[0]))).contains("def_shaders\\def_trans"));

  // A texture with no .thm.
  let unthumbed: Scene = Scene::new("refusals_thm")?;

  std::fs::remove_file(unthumbed.roots[0].join("wall.thm"))?;
  assert!(error(unthumbed.load(&Scene::faces(&[0]))).contains("such as wall"));

  // An alpha image missing, or of another size than its .thm.
  let unimaged: Scene = Scene::new("refusals_image")?;

  std::fs::remove_file(unimaged.roots[1].join("trees").join("leaf.dds"))?;
  assert!(error(unimaged.load(&Scene::faces(&[1]))).contains("trees\\leaf has no .dds"));
  Scene::write_leaf(&unimaged.roots[1].join("trees").join("leaf.dds"), 4)?;
  assert!(error(unimaged.load(&Scene::faces(&[1]))).contains("is 4 by 4, and its .thm says 8 by 4"));

  Ok(())
}
