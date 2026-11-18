use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, XrlcTracer};
use xrf_level::{BuildCformFile, BuildCformRaycastFace, BuildPrjFile};

use crate::geometry::collision::geometry_ray_model::GeometryRayModel;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::split::geometry_cut::GeometryCut;
use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::input::compiler_texture::CompilerTexture;
use crate::input::compiler_textures::CompilerTextures;
use crate::light::bake::light_bake::LightBake;
use crate::light::bake::light_bake_census::LightBakeCensus;
use crate::light::bake::light_bake_input::LightBakeInput;
use crate::light::bake::light_bake_result::LightBakeResult;
use crate::light::bake::light_baked::LightBaked;
use crate::light::implicit::implicit_lighter::ImplicitLighter;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::atlas::lightmap_atlas::LightmapAtlas;
use crate::light::lightmap::atlas::lightmap_images::LightmapImages;
use crate::light::lightmap::chart::light_chart_scene::LightChartScene;
use crate::light::lightmap::lightmap_backend::LightmapBackend;
use crate::light::lightmap::lit::light_chart_lighter::LightChartLighter;
use crate::light::lightmap::lit::lit_chart::LitChart;
use crate::light::point::light_color::LightColor;
use crate::light::source::light_sources::LightSources;
use crate::light::vertex::light_hemisphere::LightHemisphere;
use crate::light::vertex::light_model_faces::LightModelFaces;
use crate::light::vertex::light_models::LightModels;
use crate::light::vertex::light_models_input::LightModelsInput;
use crate::light::vertex::light_vertices::LightVertices;
use crate::ray::ray_scene::RayScene;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;

/// The light baker, the light step's part of a level's compile: every vertex lit ([`LightHemisphere`],
/// [`LightVertices`]), every multi-use model and placement ([`LightModels`]), the lightmapped faces cut into charts,
/// each lit and compressed ([`LightChartLighter`]) and packed onto pages ([`LightmapAtlas`]), and each implicitly lit
/// texture's map ([`ImplicitLighter`]), the lightmaps' and implicit maps' shadow rays traced by the request's backend.
///
/// The vertex and model passes trace on the CPU through xrLC's own ray model ([`XrlcTracer`]), which loses the rays
/// xrLC's tree loses, as their light is matched vertex for vertex. The lightmaps and implicit maps trace
/// through [`BvhTracer`], which the GPU backend mirrors byte for byte; their light is matched in distribution.
pub(crate) struct LightBaker;

impl LightBaker {
  /// The request's geometry lit, and its splits with each chart cut into a split of its own (`xrPhase_UVmap`).
  ///
  /// # Errors
  ///
  /// Returns an error where the shadow materials cannot be loaded, a multi-use model refuses its light, an implicitly
  /// lit texture's image is missing or unreadable, and for the GPU backend where this build has none, no GPU traces
  /// rays or the GPU fails; `Auto` lights on the CPU instead where the GPU does not start.
  pub(crate) fn bake(request: &LightBakeInput<'_>) -> XrfResult<LightBaked> {
    let started: Instant = Instant::now();
    let LightBakeInput {
      level,
      tracing,
      mesh,
      normals,
      coordinates,
      shaders,
      ray_model,
      lights,
    } = *request;
    let project: &BuildPrjFile = level.project;
    let sources: LightSources = LightSources::from_file(lights);
    let load = |faces: &[BuildCformRaycastFace]| {
      ShadowMaterials::load(&ShadowMaterialsRequest {
        project,
        faces,
        library: level.library,
        textures: level.roots,
      })
    };
    let welded: BuildCformFile = GeometryRayModel::build_welded(mesh, project, shaders)?;
    let hemispheres: Vec<f32> = LightHemisphere::of(
      mesh,
      &sources.hemi,
      &XrlcTracer::of_triangles(welded.get_triangles()),
      &load(&welded.raycast_faces)?,
    );
    // The vertex passes trace through xrLC's own ray model, the lightmaps through the tree the GPU mirrors.
    let vertex_tracer: XrlcTracer = XrlcTracer::of_triangles(ray_model.get_triangles());
    let shadows: ShadowMaterials = load(&ray_model.raycast_faces)?;
    let vertices: Vec<LightColor> =
      LightVertices::of(mesh, normals, shaders, &hemispheres, &sources, &vertex_tracer, &shadows);
    let model_faces: LightModelFaces = LightModels::get_model_faces(project, shaders)?;
    let models: LightModels = LightModels::of(&LightModelsInput {
      project,
      shaders,
      lights: &sources,
      model_shadows: &load(&model_faces.faces)?,
      model_starts: &model_faces.starts,
      tracer: &vertex_tracer,
      shadows: &shadows,
      samples: level.quality.get_model_samples(&project.params),
    })?;

    drop(vertex_tracer);

    let tracer: BvhTracer = ray_model.build_tracer();
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
    let GeometryCut { splits, charts } = GeometrySplits::cut(mesh, shaders, Some(&scene));
    let ray_faces: Vec<Option<u32>> = GeometryRayModel::get_static_faces(mesh, normals, shaders);
    let lighter: LightChartLighter<'_> = LightChartLighter {
      scene: &scene,
      lights: &sources,
      rays: RayScene {
        tracer: &tracer,
        shadows: &shadows,
      },
      ray_faces: &ray_faces,
      // `(4 + rms_zero) / 2` and `(8 + rms) / 2`.
      rms_zero: 4_u32.midpoint(project.params.lightmap_rms_zero),
      rms_shrink: 8_u32.midpoint(project.params.lightmap_rms),
    };
    let backend: LightmapBackend = LightmapBackend::start(&tracing, &lighter)?;
    let lit: Vec<LitChart> = backend.light_charts(&lighter, &charts)?;
    let atlas: LightmapAtlas = LightmapAtlas::pack(&charts, &lit, &scene);
    let lightmaps: LightmapImages = atlas.compose_all(&lit);

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

    Ok(LightBaked {
      splits,
      light: LightBakeResult {
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
    })
  }

  /// Each texture lit implicitly, its faces those of `mesh` rendered with it (`hasImplicitLighting`), in the mesh's
  /// order, its image read from the first of `roots` holding it, lit by `lighter` on `backend`.
  ///
  /// # Errors
  ///
  /// Returns an error for a texture whose image is missing or unreadable, and when the GPU refuses a batch.
  fn get_implicit_maps(
    backend: &LightmapBackend,
    lighter: &ImplicitLighter<'_>,
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
        let path: PathBuf = CompilerTextures::find(roots, &texture.name, "dds")?.ok_or_else(|| {
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
