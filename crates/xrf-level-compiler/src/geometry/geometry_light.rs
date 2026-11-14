use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, Tracer};
use xrf_level::{BuildCformFile, BuildCformRaycastFace, BuildPrjFile, LevelLightRole};

use crate::chart::implicit_lighter::ImplicitLighter;
use crate::chart::implicit_map::ImplicitMap;
use crate::chart::light_chart::LightChart;
use crate::chart::light_chart_lighter::LightChartLighter;
use crate::chart::light_chart_scene::LightChartScene;
use crate::chart::lightmap_atlas::LightmapAtlas;
use crate::chart::lightmap_backend::LightmapBackend;
use crate::chart::lightmap_faces::LightmapFaces;
use crate::chart::lit_chart::LitChart;
use crate::compiler_texture::CompilerTexture;
use crate::compiler_textures::CompilerTextures;
use crate::geometry::geometry_light_request::GeometryLightRequest;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_ray_model::GeometryRayModel;
use crate::geometry::geometry_splits::GeometrySplits;
use crate::light::light_bake::LightBake;
use crate::light::light_bake_census::LightBakeCensus;
use crate::light::light_color::LightColor;
use crate::light::light_hemisphere::LightHemisphere;
use crate::light::light_models::LightModels;
use crate::light::light_source::LightSource;
use crate::light::light_vertices::LightVertices;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;

/// A level's baked light, the light step's part of its compile: every vertex lit ([`LightHemisphere`],
/// [`LightVertices`]), every multi-use model and placement ([`LightModels`]), the lightmapped faces cut into charts,
/// each lit and compressed ([`LightChartLighter`]) and packed onto pages ([`LightmapAtlas`]), and each implicitly lit
/// texture's map ([`ImplicitLighter`]), the lightmaps' and implicit maps' shadow rays traced by the request's backend.
pub(crate) struct GeometryLight {
  /// Each split vertex's light.
  pub(crate) vertices: Vec<LightColor>,
  pub(crate) models: LightModels,
  /// Each face's lightmap page and coordinates.
  pub(crate) faces: LightmapFaces,
  /// Each lightmap page's two textures, `lmap#N_1` and `_2`, 1024 texels a side row by row ([`LightmapAtlas::compose`]).
  pub(crate) lightmaps: Vec<[Vec<[u8; 4]>; 2]>,
  /// Each texture lit implicitly, such as terrain's; none at draft build quality.
  pub(crate) implicit_maps: Vec<ImplicitMap>,
  pub(crate) bake: LightBake,
}

impl GeometryLight {
  /// The request's geometry lit, and its splits with each chart cut into a split of its own (`xrPhase_UVmap`).
  ///
  /// # Errors
  ///
  /// Returns an error where the shadow materials cannot be loaded, a multi-use model refuses its light, an implicitly
  /// lit texture's image is missing or unreadable, and for the GPU backend where this build has none, no GPU traces
  /// rays or the GPU fails; `Auto` lights on the CPU instead where the GPU does not start.
  pub(crate) fn bake(request: &GeometryLightRequest<'_>) -> XrfResult<(GeometrySplits, Self)> {
    let started: Instant = Instant::now();
    let GeometryLightRequest {
      level,
      bake,
      mesh,
      normals,
      coordinates,
      shaders,
      ray_model,
      lights,
    } = *request;
    let project: &BuildPrjFile = level.project;
    let sources: [Vec<LightSource>; 3] =
      LevelLightRole::ALL.map(|role| lights.get_role_lights(role).map(LightSource::from).collect());
    let load = |faces: &[BuildCformRaycastFace]| {
      ShadowMaterials::load(&ShadowMaterialsRequest {
        project,
        faces,
        library: level.library,
        textures: level.roots,
      })
    };
    let welded: BuildCformFile = GeometryRayModel::build_welded(mesh, project, shaders)?;
    let hemispheres: Vec<f32> =
      LightHemisphere::of(mesh, &sources[1], &welded.build_tracer(), &load(&welded.raycast_faces)?);
    let tracer: BvhTracer = ray_model.build_tracer();
    let shadows: ShadowMaterials = load(&ray_model.raycast_faces)?;
    let vertices: Vec<LightColor> =
      LightVertices::of(mesh, normals, shaders, &hemispheres, &sources, &tracer, &shadows);
    let (model_faces, model_starts) = LightModels::get_model_faces(project, shaders)?;
    let models: LightModels = LightModels::of(
      project,
      shaders,
      &sources,
      &load(&model_faces)?,
      &model_starts,
      &tracer,
      &shadows,
      level.quality.is_draft(project.params.is_draft()),
    )?;
    let scene: LightChartScene<'_> = LightChartScene::of(
      mesh,
      normals,
      project,
      shaders,
      level.textures,
      level
        .quality
        .get_pixels_per_meter(project.params.lightmap_pixels_per_meter),
      level.quality.get_jitter_samples(project.params.lightmap_jitter_samples),
    );
    let (splits, charts): (GeometrySplits, Vec<LightChart>) = GeometrySplits::cut(mesh, shaders, Some(&scene));
    let ray_faces: Vec<Option<u32>> = GeometryRayModel::get_static_faces(mesh, normals, shaders);
    let lighter: LightChartLighter<'_, BvhTracer> = LightChartLighter {
      scene: &scene,
      lights: &sources,
      tracer: &tracer,
      shadows: &shadows,
      ray_faces: &ray_faces,
      // `(4 + rms_zero) / 2` and `(8 + rms) / 2`.
      rms_zero: 4_u32.midpoint(project.params.lightmap_rms_zero),
      rms_shrink: 8_u32.midpoint(project.params.lightmap_rms),
    };
    let backend: LightmapBackend = LightmapBackend::start(bake.backend, bake.dxc, bake.gpu, &tracer, &shadows)?;
    let lit: Vec<LitChart> = backend.light_charts(&lighter, &charts)?;
    let atlas: LightmapAtlas = LightmapAtlas::pack(&charts, &lit, &scene);
    let lightmaps: Vec<[Vec<[u8; 4]>; 2]> = (0..atlas.pages)
      .into_par_iter()
      .map(|page| atlas.compose(page, &lit))
      .collect();

    drop(lit);

    // xrLC lights no texture implicitly at draft quality (`ImplicitLighting`).
    let implicit_maps: Vec<ImplicitMap> = if project.params.is_draft() {
      Vec::new()
    } else {
      Self::get_implicit_maps(
        &backend,
        &ImplicitLighter {
          lighter: &lighter,
          coordinates: &coordinates.faces,
        },
        mesh,
        project,
        level.textures,
        level.roots,
      )?
    };
    let census: LightBakeCensus = LightBakeCensus {
      charts: charts.len(),
      lightmaps: atlas.pages,
      implicit_maps: implicit_maps.len(),
    };

    Ok((
      splits,
      Self {
        vertices,
        models,
        faces: atlas.get_faces(&charts, mesh.faces.len()),
        lightmaps,
        implicit_maps,
        bake: LightBake {
          census,
          gpu: backend.get_name(),
          duration: started.elapsed(),
        },
      },
    ))
  }

  /// Each texture lit implicitly, its faces those of `mesh` rendered with it (`hasImplicitLighting`), in the mesh's
  /// order, its image read from the first of `roots` holding it, lit by `lighter` on `backend`.
  ///
  /// # Errors
  ///
  /// Returns an error for a texture whose image is missing or unreadable, and when the GPU refuses a batch.
  fn get_implicit_maps<T: Tracer + Sync>(
    backend: &LightmapBackend,
    lighter: &ImplicitLighter<'_, T>,
    mesh: &GeometryMesh,
    project: &BuildPrjFile,
    textures: &[CompilerTexture],
    roots: &[PathBuf],
  ) -> XrfResult<Vec<ImplicitMap>> {
    let mut faces: BTreeMap<u16, Vec<u32>> = BTreeMap::new();

    for (index, face) in mesh.faces.iter().enumerate() {
      let Some(material) = project.materials.get(usize::from(face.material)) else {
        continue;
      };
      let is_rendered: bool = lighter
        .lighter
        .scene
        .is_rendered
        .get(usize::from(face.material))
        .copied()
        .unwrap_or(false);

      if is_rendered
        && textures
          .get(usize::from(material.texture))
          .is_some_and(|texture| texture.is_implicitly_lit)
      {
        faces.entry(material.texture).or_default().push(index as u32);
      }
    }

    faces
      .into_iter()
      .map(|(texture, faces)| {
        let texture: &CompilerTexture = &textures[usize::from(texture)];
        let path: PathBuf = CompilerTextures::find(roots, &texture.name, "dds").ok_or_else(|| {
          XrfError::new_not_found_error(format!(
            "The implicitly lit texture {} has no .dds under the texture roots",
            texture.name
          ))
        })?;
        let image: RgbaImage = DdsFile::read_from_path(&path)?.decode_rgba(0)?;
        let (width, height) = image.dimensions();
        let base: Vec<[u8; 4]> = image.pixels().map(|pixel| pixel.0).collect();

        backend.light_implicit(lighter, texture.name.clone(), base, width, height, &faces)
      })
      .collect()
  }
}
