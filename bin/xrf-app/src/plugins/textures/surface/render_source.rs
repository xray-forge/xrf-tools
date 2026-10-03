use std::collections::HashMap;

use xrf_dds::{DdsEncoding, DdsFile, DdsMetadata, DdsMipChain, DdsMipmaps, ImageFormat, Quality, RgbaImage};
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::{XrayMaterialDescriptor, XraySurfaceDeclaration, XraySurfaceDescriptor};
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelSource, RenderLevelSpawn, RenderLevelWeather, RenderSpawnCategory,
  RenderSpawnLighting, RenderSpawnModel, RenderSpawnModels, RenderSpawnObject,
};
use xrf_vfs::{XrayAsset, XrayRoots};
use xrf_visual::{LightsDescription, SectorPackage, VisualMesh, VisualPacker, VisualShape};

use crate::core::assets::AssetMountState;
use crate::plugins::textures::description::TextureDescription;
use crate::plugins::textures::file_stamp::TextureFileStamp;
use crate::plugins::textures::surface::texture_surface_alpha::TextureSurfaceAlpha;
use crate::plugins::textures::surface::texture_surface_request::TextureSurfaceRequest;
use crate::plugins::textures::surface::texture_surface_shape::TextureSurfaceShape;

/// The one object the body is: the texture's body at the origin.
const BODY_OBJECT: u32 = 0;

/// How large each body is drawn, chosen so all three frame alike under one camera.
const SHAPE_EXTENT: f32 = 2.0;

/// How thick the flat body is, as a fraction of its extent: enough to see it turn, too little to read as a box.
const SLAB_THICKNESS: f32 = 0.02;

/// The box face the plane's texture is laid on: `+z`, the one facing the camera.
const PLANE_FACE: usize = 4;

/// A texture laid on a body as the native renderer draws it: a scene of one object and no level, the texture and its
/// bump pair read from the files its description located.
pub struct TextureRenderSource {
  assets: AssetMountState,
  roots: XrayRoots,
  /// The texture's reference, which its face names.
  reference: String,
  material: Option<XrayMaterialDescriptor>,
  /// Every texture the body samples, by its reference, as the file it was located at; `None` for one located nowhere.
  files: HashMap<String, Option<XrayAsset>>,
  /// The roots and what each file looked like when the body was asked for, so an edited file is read again.
  scope: String,
  shape: TextureSurfaceShape,
  tiling: f32,
  alpha: TextureSurfaceAlpha,
  aspect: f32,
}

impl TextureRenderSource {
  pub fn new(description: &TextureDescription, request: &TextureSurfaceRequest, assets: AssetMountState) -> Self {
    let mut files: HashMap<String, Option<XrayAsset>> =
      HashMap::from([(description.reference.clone(), description.texture.clone())]);

    if let Some(bump) = description
      .material
      .as_ref()
      .and_then(|material| material.bump.as_ref())
    {
      for input in [&bump.bump, &bump.companion] {
        files.insert(input.reference.clone(), input.resolution.get_asset().cloned());
      }
    }

    let mut stamps: Vec<(String, Option<TextureFileStamp>)> = files
      .values()
      .flatten()
      .map(|asset| {
        let stamp: Option<TextureFileStamp> = asset
          .to_physical_path()
          .and_then(|path| TextureFileStamp::read(&path).ok().flatten());

        (asset.get_logical_path().as_str().to_owned(), stamp)
      })
      .collect();

    stamps.sort_by(|a, b| a.0.cmp(&b.0));

    Self {
      assets,
      scope: format!("{:?} {stamps:?}", description.roots),
      roots: description.roots.clone(),
      reference: description.reference.clone(),
      material: description.material.clone(),
      files,
      shape: request.shape,
      tiling: request.tiling,
      alpha: request.alpha,
      aspect: request.aspect,
    }
  }

  /// The body's meshes, each beside the surface it draws with: the texture's face, or the plain edge of a slab.
  fn build_meshes(&self) -> Vec<(VisualMesh, XraySurfaceDescriptor)> {
    let face = |mesh: VisualMesh| (self.dress(mesh), self.get_face_surface());
    let edge = |mesh: VisualMesh| {
      (
        mesh,
        XraySurfaceDescriptor::opaque(None, XraySurfaceDeclaration::Undeclared),
      )
    };

    match self.shape {
      TextureSurfaceShape::Plane => VisualShape::create_box(SHAPE_EXTENT, SHAPE_EXTENT, SHAPE_EXTENT * SLAB_THICKNESS)
        .into_iter()
        .enumerate()
        .map(|(index, mesh)| if index == PLANE_FACE { face(mesh) } else { edge(mesh) })
        .collect(),
      TextureSurfaceShape::Sphere => vec![face(VisualShape::create_sphere(SHAPE_EXTENT / 2.0, 96, 64))],
      TextureSurfaceShape::Cube => VisualShape::create_box(SHAPE_EXTENT * 0.8, SHAPE_EXTENT * 0.8, SHAPE_EXTENT * 0.8)
        .into_iter()
        .map(face)
        .collect(),
    }
  }

  /// A mesh naming the texture, repeated across it as many times as asked.
  fn dress(&self, mut mesh: VisualMesh) -> VisualMesh {
    mesh.texture_name = Some(self.reference.clone());
    mesh.uvs = mesh
      .uvs
      .iter()
      .map(|[u, v]| [u * self.tiling, v * self.tiling])
      .collect();
    mesh
  }

  /// The surface the texture's face draws with: its alpha read as asked, its bump pair and lighting model as its
  /// descriptor declares them.
  fn get_face_surface(&self) -> XraySurfaceDescriptor {
    XraySurfaceDescriptor {
      textures: vec![self.reference.clone()],
      draw: self.alpha.to_draw(),
      bump: self.material.as_ref().and_then(|material| material.bump.clone()),
      material: self
        .material
        .as_ref()
        .map_or(XrayMaterialDescriptor::DEFAULT_MATERIAL, |material| material.material),
      ..XraySurfaceDescriptor::opaque(None, XraySurfaceDeclaration::Undeclared)
    }
  }

  /// Where the body stands: at the origin, the plane stretched to the texture's proportions.
  fn get_transform(&self) -> [f32; 16] {
    let (width, height): (f32, f32) = match self.shape {
      TextureSurfaceShape::Plane if self.aspect >= 1.0 => (1.0, 1.0 / self.aspect),
      TextureSurfaceShape::Plane if self.aspect > 0.0 => (self.aspect, 1.0),
      _ => (1.0, 1.0),
    };

    [
      width, 0.0, 0.0, 0.0, 0.0, height, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
  }
}

impl RenderAssetSource for TextureRenderSource {
  fn get_texture_scope(&self) -> String {
    self.scope.clone()
  }

  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let Some(Some(asset)) = self.files.get(reference) else {
      return Ok(None);
    };
    let bytes: Vec<u8> = self
      .assets
      .with_probe(&self.roots, |probe| probe.read_asset_bytes(asset))
      .map_err(XrfError::new_asset_error)??;

    to_flat_texture(bytes).map(Some)
  }
}

impl RenderLevelSource for TextureRenderSource {
  fn get_sector_count(&self) -> u32 {
    0
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    Err(XrfError::new_not_found_error(format!(
      "A texture has no sector {sector}"
    )))
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    Ok(LightsDescription::default())
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    Ok(RenderLevelWeather::default())
  }

  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(XrfError::new_not_found_error(format!(
      "A texture plays no weather '{name}'"
    )))
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    Ok(RenderLevelSpawn {
      visuals: vec![self.reference.clone()],
      objects: vec![RenderSpawnObject {
        index: BODY_OBJECT,
        category: RenderSpawnCategory::Props,
        is_released: false,
        visual: 0,
        transform: self.get_transform(),
      }],
      detail: 0.0,
    })
  }

  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels> {
    let (meshes, surfaces): (Vec<VisualMesh>, Vec<XraySurfaceDescriptor>) = self.build_meshes().into_iter().unzip();

    Ok(RenderSpawnModels {
      models: names
        .iter()
        .map(|name| RenderSpawnModel {
          name: name.clone(),
          package: VisualPacker::pack_meshes(&meshes),
          surfaces: surfaces.clone(),
          skeleton: None,
        })
        .collect(),
      // Open sky on every side: no level stands around it to shade its hemisphere.
      lighting: vec![RenderSpawnLighting {
        object: BODY_OBJECT,
        cube: [1.0; 6],
        sky: 1.0,
      }],
      failures: Vec::new(),
    })
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }
}

/// A texture file as a flat picture the body can be dressed in: as stored, or a cube's or a volume's faces unfolded
/// into one eight-bit picture, as the textures explorer shows them.
fn to_flat_texture(bytes: Vec<u8>) -> XrfResult<Vec<u8>> {
  let metadata: DdsMetadata = DdsFile::read_metadata_from_bytes(&bytes)?;

  if !metadata.is_cubemap && !metadata.is_volume {
    return Ok(bytes);
  }

  let picture: RgbaImage = DdsFile::read_from_bytes(&bytes)?.decode_rgba(0)?;

  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
    .encode(&DdsMipChain::build(&picture, DdsMipmaps::Disabled)?)?
    .write_to_bytes()
}
