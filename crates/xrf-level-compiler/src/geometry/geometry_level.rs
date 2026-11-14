use std::collections::BTreeMap;
use std::path::PathBuf;

use rayon::prelude::*;
use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, ProgressiveWindow, Sky, SkyLight, Tracer};
use xrf_level::{
  BuildCformFile, BuildCformRaycastFace, BuildPrjFile, LevelBundle, LevelCformFile, LevelCformGeometry,
  LevelCformHeader, LevelDynamicLight, LevelGeomBuffers, LevelGeomPacker, LevelGeomSlideWindowItem, LevelGlow,
  LevelHeaderChunk, LevelLight, LevelLightRole, LevelLightsChunk, LevelLightsFile, LevelPortal, LevelSector,
  LevelShaderEntry, LevelShadersChunk, LevelVisualRecord,
};
use xrf_math::{Color, Sphere, Vector3d};
use xrf_ogf::{
  OgfFastpathChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfLodDefinitionChunk, OgfLodFacet, OgfLodVertex,
  OgfModelType, OgfSlideWindow, OgfSwiContainerChunk, OgfSwiDataChunk, OgfTreeColor, OgfTreeDefinitionChunk,
};

use crate::chart::implicit_lighter::ImplicitLighter;
use crate::chart::implicit_map::ImplicitMap;
use crate::chart::light_chart::LightChart;
use crate::chart::light_chart_lighter::LightChartLighter;
use crate::chart::light_chart_scene::LightChartScene;
use crate::chart::lightmap_atlas::LightmapAtlas;
use crate::chart::lightmap_backend::LightmapBackend;
use crate::chart::lightmap_faces::LightmapFaces;
use crate::chart::lit_chart::LitChart;
use crate::compiler_shaders::CompilerShaders;
use crate::compiler_texture::CompilerTexture;
use crate::compiler_textures::CompilerTextures;
use crate::geometry::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::geometry_bounds::GeometryBounds;
use crate::geometry::geometry_collision::GeometryCollision;
use crate::geometry::geometry_hierarchy::GeometryHierarchy;
use crate::geometry::geometry_level_census::GeometryLevelCensus;
use crate::geometry::geometry_level_request::GeometryLevelRequest;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_models::GeometryModels;
use crate::geometry::geometry_node_kind::GeometryNodeKind;
use crate::geometry::geometry_normals::GeometryNormals;
use crate::geometry::geometry_packed_visual::GeometryPackedVisual;
use crate::geometry::geometry_ray_model::GeometryRayModel;
use crate::geometry::geometry_shader_table::GeometryShaderTable;
use crate::geometry::geometry_splits::GeometrySplits;
use crate::geometry::geometry_tangents::GeometryTangents;
use crate::geometry::geometry_visual::GeometryVisual;
use crate::geometry::geometry_visuals::GeometryVisuals;
use crate::geometry::geometry_visuals_request::GeometryVisualsRequest;
use crate::light::light_color::LightColor;
use crate::light::light_hemisphere::LightHemisphere;
use crate::light::light_models::LightModels;
use crate::light::light_quality::LightQuality;
use crate::light::light_source::LightSource;
use crate::light::light_vertices::LightVertices;
use crate::shadow::shadow_materials::ShadowMaterials;
use crate::shadow::shadow_materials_request::ShadowMaterialsRequest;

/// Everything xrLC writes for a level's geometry, compiled from its `build.prj`, its vertices and trees lit: the `level`
/// file, its render geometry, the game's collision form, the compilers' ray model and lights, and the lightmap pages.
#[derive(Debug)]
pub struct GeometryLevel {
  pub bundle: LevelBundle,
  /// `level.geom`.
  pub geometry: LevelGeomBuffers,
  /// `level.geomx`.
  pub fast: LevelGeomBuffers,
  /// `level.cform`.
  pub collision: LevelCformGeometry,
  /// `build.cform`.
  pub ray_model: BuildCformFile,
  /// `build.lights`.
  pub lights: LevelLightsFile,
  /// Each lightmap page's two textures, `lmap#N_1` and `_2`, 1024 texels a side row by row ([`LightmapAtlas::compose`]).
  pub lightmaps: Vec<[Vec<[u8; 4]>; 2]>,
  /// Each texture lit implicitly, such as terrain's; none at draft quality.
  pub implicit_maps: Vec<ImplicitMap>,
  /// What the compile made, for its report.
  pub census: GeometryLevelCensus,
  /// The GPU that traced the lightmaps' shadow rays, none where the CPU did.
  pub gpu: Option<String>,
}

impl GeometryLevel {
  /// `XRCL_PRODUCTION_VERSION`, the version the engine asserts a level carries.
  pub const VERSION: u16 = 14;

  /// The level the request's project compiles to, its compiler shaders from its library and its textures as it loads
  /// them, their alpha images under its roots dimming the light, lit at its quality: every vertex lit
  /// ([`LightHemisphere`], [`LightVertices`]), every multi-use model and placement ([`LightModels`]), the lightmapped
  /// faces cut into charts, each lit and compressed ([`LightChartLighter`]), and packed onto pages ([`LightmapAtlas`]),
  /// the lightmaps' and implicit maps' shadow rays traced by its backend.
  ///
  /// # Errors
  ///
  /// Returns an error where a phase refuses the project, for a static light layer `build.prj` lacks, where the shadow
  /// materials cannot be loaded, and for the GPU backend where this build has none, no GPU traces rays or the GPU
  /// fails; `Auto` lights on the CPU instead where the GPU does not start.
  pub fn compile(request: &GeometryLevelRequest<'_>) -> XrfResult<Self> {
    let GeometryLevelRequest {
      project,
      library,
      textures,
      roots,
      quality,
      ..
    } = *request;
    let shaders: CompilerShaders = CompilerShaders::resolve(project, library)?;
    let mut mesh: GeometryMesh = GeometryMesh::load(project)?;

    mesh.optimize(project.params.weld_distance)?;
    mesh.tessellate();

    let normals: GeometryNormals = GeometryNormals::of(&mesh);
    let collision: LevelCformGeometry = GeometryCollision::build(&mesh, project, &shaders)?;
    let coordinates: GeometryBaseCoordinates = GeometryBaseCoordinates::of(&mesh, &normals);
    let tangents: GeometryTangents = GeometryTangents::of(&mesh, &normals, &coordinates);
    let ray_model: BuildCformFile = GeometryRayModel::build(&mesh, &normals, &coordinates, project, &shaders)?;
    let lights: LevelLightsFile = Self::get_lights(project, quality)?;
    let sources: [Vec<LightSource>; 3] =
      LevelLightRole::ALL.map(|role| lights.get_role_lights(role).map(LightSource::from).collect());
    let load = |faces: &[BuildCformRaycastFace]| {
      ShadowMaterials::load(&ShadowMaterialsRequest {
        project,
        faces,
        library,
        textures: roots,
      })
    };
    let welded: BuildCformFile = GeometryRayModel::build_welded(&mesh, project, &shaders)?;
    let hemispheres: Vec<f32> = LightHemisphere::of(
      &mesh,
      &sources[1],
      &welded.build_tracer(),
      &load(&welded.raycast_faces)?,
    );
    let tracer: BvhTracer = ray_model.build_tracer();
    let shadows: ShadowMaterials = load(&ray_model.raycast_faces)?;
    let vertex_lights: Vec<LightColor> =
      LightVertices::of(&mesh, &normals, &shaders, &hemispheres, &sources, &tracer, &shadows);
    let (model_faces, model_starts) = LightModels::get_model_faces(project, &shaders)?;
    let model_lights: LightModels = LightModels::of(
      project,
      &shaders,
      &sources,
      &load(&model_faces)?,
      &model_starts,
      &tracer,
      &shadows,
      quality.is_draft(project.params.is_draft()),
    )?;
    let scene: LightChartScene<'_> = LightChartScene::of(
      &mesh,
      &normals,
      project,
      &shaders,
      textures,
      quality.get_pixels_per_meter(project.params.lightmap_pixels_per_meter),
      quality.get_jitter_samples(project.params.lightmap_jitter_samples),
    );
    let (mut splits, charts): (GeometrySplits, Vec<LightChart>) = GeometrySplits::cut(&mesh, &shaders, Some(&scene));
    let ray_faces: Vec<Option<u32>> = GeometryRayModel::get_static_faces(&mesh, &normals, &shaders);
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
    let backend: LightmapBackend =
      LightmapBackend::start(request.backend, request.dxc, request.gpu, &tracer, &shadows)?;
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
        &mesh,
        project,
        textures,
        roots,
      )?
    };
    let faces: LightmapFaces = atlas.get_faces(&charts, mesh.faces.len());

    splits.merge(&mesh, Some(&faces.pages));

    let visuals: GeometryVisuals = GeometryVisuals::build(&GeometryVisualsRequest {
      mesh: &mesh,
      normals: &normals,
      coordinates: &coordinates,
      tangents: &tangents,
      splits: &splits,
      project,
      shaders: &shaders,
      textures,
      lights: Some(&vertex_lights),
      lightmaps: Some(&faces),
    })?;
    let models: GeometryModels = GeometryModels::build(project, &shaders, textures, Some(&model_lights))?;
    let hierarchy: GeometryHierarchy = GeometryHierarchy::build(&visuals, &models, project)?;
    let header: LevelHeaderChunk = LevelHeaderChunk {
      xrlc_version: Self::VERSION,
      xrlc_quality: project.params.quality,
    };
    let mut geometry: LevelGeomPacker = LevelGeomPacker::default();
    let mut fast: LevelGeomPacker = LevelGeomPacker::default();
    let mut slide_windows: Vec<LevelGeomSlideWindowItem> = Vec::new();
    let packed_models: Vec<Vec<GeometryPackedVisual>> = models.pack(&mut geometry, &mut slide_windows)?;
    let packed_visuals: Vec<GeometryPackedVisual> = visuals.pack(&mut geometry, &mut fast)?;
    let mut table: GeometryShaderTable = GeometryShaderTable::default();

    // A dumb first entry (`Build.cpp:177`), then the glows' (`Build.cpp:367`), then the visuals' as they are saved.
    table.register("");

    let glows: Vec<LevelGlow> = project
      .glows
      .iter()
      .map(|glow| {
        Ok(LevelGlow {
          position: glow.position,
          radius: glow.size,
          shader: table.register(&Self::get_dress(project, textures, glow.material)?),
        })
      })
      .collect::<XrfResult<_>>()?;
    let mut records: Vec<LevelVisualRecord> = Vec::with_capacity(hierarchy.nodes.len());

    for node in &hierarchy.nodes {
      let bounds: &GeometryBounds = &node.bounds;
      let header = |model_type: OgfModelType, shader_id: u16| OgfHeaderChunk {
        version: 4,
        model_type: model_type.to_raw(),
        shader_id,
        bounding_box: bounds.bounds,
        bounding_sphere: Sphere {
          center: bounds.center,
          radius: bounds.radius,
        },
      };

      records.push(match &node.kind {
        GeometryNodeKind::Visual(index) => {
          let visual = &visuals.visuals[*index as usize];
          let placed: &GeometryPackedVisual = &packed_visuals[*index as usize];

          let model_type: OgfModelType = if visual.windows.is_some() {
            OgfModelType::Progressive
          } else {
            OgfModelType::Normal
          };

          LevelVisualRecord {
            header: header(model_type, table.register(&visual.shader)),
            texture: None,
            geometry: Some(Self::get_container(placed, visual.vertices.len(), visual.faces.len())),
            swi: visual.windows.as_deref().map(Self::get_windows),
            fastpath: placed
              .fast_vertices
              .zip(placed.fast_indices)
              .map(|(vertices, indices)| OgfFastpathChunk {
                geometry: OgfGeometryContainerChunk {
                  vertex_buffer_id: vertices.buffer,
                  vertex_base: vertices.start,
                  vertex_count: visual.fast_vertices.len() as u32,
                  index_buffer_id: indices.buffer,
                  index_base: indices.start,
                  index_count: visual.fast_faces.len() as u32 * 3,
                },
                swi: visual.fast_windows.as_deref().map(Self::get_windows),
              }),
            tree: None,
            swi_container: None,
            children: None,
            lod: None,
          }
        }
        GeometryNodeKind::Tree(index) => {
          let tree = &models.trees[*index as usize];
          let visual = &models.models[tree.model as usize].visuals[tree.visual as usize];
          let color = |terms: [f32; 5]| OgfTreeColor {
            rgb: Vector3d::new(terms[0], terms[1], terms[2]),
            hemi: terms[3],
            sun: terms[4],
          };

          let placed: &GeometryPackedVisual = &packed_models[tree.model as usize][tree.visual as usize];
          let model_type: OgfModelType = if placed.slide_window.is_some() {
            OgfModelType::TreePm
          } else {
            OgfModelType::TreeSt
          };

          LevelVisualRecord {
            header: header(model_type, table.register(&visual.shader)),
            texture: None,
            geometry: Some(Self::get_container(placed, visual.vertices.len(), visual.faces.len())),
            swi: None,
            fastpath: None,
            tree: Some(OgfTreeDefinitionChunk {
              transform: tree.transform,
              scale: color(tree.scale),
              bias: color(tree.bias),
            }),
            swi_container: placed
              .slide_window
              .map(|index| OgfSwiContainerChunk { ext_swib_index: index }),
            children: None,
            lod: None,
          }
        }
        GeometryNodeKind::Lod(index) => {
          let lod = &models.lods[*index as usize];
          let first_tree: usize = hierarchy
            .nodes
            .iter()
            .position(|node| matches!(node.kind, GeometryNodeKind::Tree(tree) if Some(&tree) == lod.trees.first()))
            .ok_or_else(|| XrfError::new_unexpected_error("A LOD stands over no tree node"))?;
          let material: u16 = u16::try_from(lod.material)
            .map_err(|_| XrfError::new_invalid_error(format!("A LOD names material {}", lod.material)))?;

          LevelVisualRecord {
            header: header(
              OgfModelType::Lod,
              table.register(&Self::get_dress(project, textures, u32::from(material))?),
            ),
            texture: None,
            geometry: None,
            swi: None,
            fastpath: None,
            tree: None,
            swi_container: None,
            children: Some((first_tree as u32..(first_tree + lod.trees.len()) as u32).collect()),
            lod: Some(OgfLodDefinitionChunk {
              facets: lod.faces.map(|face| OgfLodFacet {
                vertices: face.map(|vertex| OgfLodVertex {
                  position: vertex.position,
                  texture_coordinate: vertex.texture_coordinate,
                  rgb_hemi: u32::from_le_bytes([vertex.light[2], vertex.light[1], vertex.light[0], vertex.light[3]]),
                  sun: vertex.sun,
                  padding: [0; OgfLodVertex::PADDING],
                }),
              }),
            }),
          }
        }
        GeometryNodeKind::Hierarchy(children) => LevelVisualRecord {
          header: header(OgfModelType::Hierarchy, 0),
          texture: None,
          geometry: None,
          swi: None,
          fastpath: None,
          tree: None,
          swi_container: None,
          children: Some(children.clone()),
          lod: None,
        },
      });
    }

    let census: GeometryLevelCensus = GeometryLevelCensus {
      visuals: visuals.visuals.len(),
      triangles: visuals.visuals.iter().map(GeometryVisual::get_triangle_count).sum(),
      progressive: visuals
        .visuals
        .iter()
        .chain(models.models.iter().flat_map(|model| &model.visuals))
        .filter(|visual| visual.windows.is_some())
        .count(),
      trees: models.trees.len(),
      sectors: hierarchy.sectors.len(),
      charts: charts.len(),
      lightmaps: atlas.pages,
    };

    Ok(Self {
      bundle: LevelBundle {
        header: header.clone(),
        portals: Some(
          project
            .portals
            .iter()
            .map(|portal| LevelPortal {
              sector_front: portal.sector_front,
              sector_back: portal.sector_back,
              vertices: portal.vertex_slots.to_vec(),
              vertex_count: portal.vertex_count,
            })
            .collect(),
        ),
        lights: Some(project.dynamic_lights.iter().map(|light| light.light.clone()).collect()),
        glows: Some(glows),
        visuals: Some(records),
        shaders: Some(LevelShadersChunk {
          entries: table
            .get_entries()
            .iter()
            .map(|entry| LevelShaderEntry::parse(entry))
            .collect(),
        }),
        sectors: Some(
          hierarchy
            .sectors
            .iter()
            .map(|sector| LevelSector {
              portals: sector.portals.clone(),
              root: sector.root,
            })
            .collect(),
        ),
      },
      geometry: geometry.into_buffers(header.clone(), slide_windows),
      fast: fast.into_buffers(header, Vec::new()),
      collision,
      ray_model,
      lights,
      lightmaps,
      implicit_maps,
      census,
      gpu: backend.get_name(),
    })
  }

  /// The `level.cform` file of the collision form.
  ///
  /// # Errors
  ///
  /// Returns an error for a form of no vertex.
  pub fn get_collision_file(&self) -> XrfResult<LevelCformFile> {
    Ok(LevelCformFile {
      header: LevelCformHeader::of(&self.collision)?,
    })
  }

  /// Each texture lit implicitly, its faces those of `mesh` rendered with it (`hasImplicitLighting`), in the mesh's
  /// order, its image read from the first of `roots` holding it, lit by `lighter`.
  ///
  /// # Errors
  ///
  /// Returns an error for a texture whose image is missing or unreadable.
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

  /// A material's shader table entry, `<render shader>/<texture>`, as glows and LODs are dressed.
  fn get_dress(project: &BuildPrjFile, textures: &[CompilerTexture], material: u32) -> XrfResult<String> {
    let entry = project
      .materials
      .get(material as usize)
      .ok_or_else(|| XrfError::new_invalid_error(format!("Material {material} is not in build.prj")))?;
    let shader = project.render_shaders.get(usize::from(entry.shader)).ok_or_else(|| {
      XrfError::new_invalid_error(format!("Material {material} names render shader {}", entry.shader))
    })?;
    let texture: &CompilerTexture = textures
      .get(usize::from(entry.texture))
      .ok_or_else(|| XrfError::new_invalid_error(format!("Material {material} names texture {}", entry.texture)))?;

    Ok(format!("{}/{}", shader.name.value, texture.name))
  }

  /// A progressive visual's windows as `OGF_SWIDATA` holds them.
  fn get_windows(windows: &[ProgressiveWindow]) -> OgfSwiDataChunk {
    OgfSwiDataChunk {
      reserved: [0; 4],
      windows: windows
        .iter()
        .map(|window| OgfSlideWindow {
          offset: window.offset,
          num_tris: window.triangles,
          num_verts: window.vertices,
        })
        .collect(),
    }
  }

  /// A visual's geometry container: where its vertices and indices were packed, and how many.
  fn get_container(placed: &GeometryPackedVisual, vertices: usize, faces: usize) -> OgfGeometryContainerChunk {
    OgfGeometryContainerChunk {
      vertex_buffer_id: placed.vertices.buffer,
      vertex_base: placed.vertices.start,
      vertex_count: vertices as u32,
      index_buffer_id: placed.indices.buffer,
      index_base: placed.indices.start,
      index_count: faces as u32 * 3,
    }
  }

  /// `build.lights` (`Build.cpp:180`): the `$static`, `$hemi` and `$sun` layers' lights as xrLC loads them
  /// (`Build_Load.cpp:211`), chunks 0, 1 and 2, the sky and the sun regenerated where `quality` asks for another
  /// quality than `build.prj`'s ([`Self::get_sky_layer`]).
  pub(crate) fn get_lights(project: &BuildPrjFile, quality: LightQuality) -> XrfResult<LevelLightsFile> {
    let layer = |name: &str| -> XrfResult<(u32, Vec<LevelDynamicLight>)> {
      let control: u32 = project
        .light_controls
        .iter()
        .position(|control| control.name.value.eq_ignore_ascii_case(name))
        .and_then(|control| u32::try_from(control).ok())
        .ok_or_else(|| XrfError::new_invalid_error(format!("build.prj has no {name} light layer")))?;
      let lights: Vec<LevelDynamicLight> = project
        .static_lights
        .iter()
        .filter(|light| light.controller == control)
        .cloned()
        .collect();

      if lights.is_empty() {
        return Err(XrfError::new_invalid_error(format!(
          "build.prj's {name} light layer holds no light"
        )));
      }

      Ok((control, lights))
    };
    let (_, static_lights) = layer("$static")?;
    let (hemi_control, mut hemi) = layer("$hemi")?;
    let (sun_control, mut sun) = layer("$sun")?;

    if let Some(sky) = quality.get_sky_quality() {
      let (pitch, heading) = Sky::get_pitch_heading(&Self::get_sun_direction(project, sun_control, &sun));

      hemi = Self::get_sky_layer(hemi, hemi_control, Sky::get_hemisphere(sky));
      sun = Self::get_sky_layer(sun, sun_control, Sky::get_sun(sky, pitch, heading)?);
    }

    let chunk = |id: u32, lights: &[LevelDynamicLight]| LevelLightsChunk::Lights {
      id,
      lights: lights.iter().map(Self::get_light).collect(),
    };

    Ok(LevelLightsFile {
      chunks: vec![chunk(0, &static_lights), chunk(1, &hemi), chunk(2, &sun)],
    })
  }

  /// A sky or sun layer at the quality `generated` holds. The editor generates the layer's directional lights and
  /// takes the scene's lights under the layer as point lights alone (`LevelBuildLights`), so the directional ones are
  /// replaced by `generated` under `controller` and the scene's kept, unless they are of that quality already (as many).
  fn get_sky_layer(held: Vec<LevelDynamicLight>, controller: u32, generated: Vec<SkyLight>) -> Vec<LevelDynamicLight> {
    if held.iter().filter(|light| light.is_sun()).count() == generated.len() {
      return held;
    }

    generated
      .iter()
      .map(|light| {
        LevelDynamicLight::new_directional(
          controller,
          Color::new(light.energy, light.energy, light.energy, 1.0),
          light.direction,
        )
      })
      .chain(held.into_iter().filter(|light| !light.is_sun()))
      .collect()
  }

  /// The way the sun's light travels: the directional dynamic light under `$sun` the editor writes along it, else the
  /// mean of the layer's directional `lights`, which spread three degrees about it.
  fn get_sun_direction(project: &BuildPrjFile, control: u32, lights: &[LevelDynamicLight]) -> Vector3d {
    if let Some(sun) = project
      .dynamic_lights
      .iter()
      .map(|light| &light.light)
      .find(|light| light.controller == control && light.is_sun())
    {
      return sun.direction;
    }

    lights
      .iter()
      .filter(|light| light.is_sun())
      .fold(Vector3d::ZERO, |sum, light| {
        Vector3d::new(
          sum.x + light.direction.x,
          sum.y + light.direction.y,
          sum.z + light.direction.z,
        )
      })
      .normalize_or_zero()
  }

  /// One static light as xrLC loads it (`R_Light`): its energy the mean of its diffuse, its colour divided by it, its
  /// range a tenth longer, and its falloff from the attenuation at that range.
  fn get_light(light: &LevelDynamicLight) -> LevelLight {
    // xrLC's x87 arithmetic leaves the energy and colour a bit or two apart from these in their last place.
    let energy: f32 = (light.diffuse.r + light.diffuse.g + light.diffuse.b) / 3.0;
    let (diffuse, energy): (Vector3d, f32) = if energy.abs() > xrf_math::EPS_S {
      (
        Vector3d::new(
          light.diffuse.r / energy,
          light.diffuse.g / energy,
          light.diffuse.b / energy,
        ),
        energy,
      )
    } else {
      (Vector3d::ZERO, 0.0)
    };
    // xrLC flushes a subnormal result to zero, which a directional light's tiny range gives.
    let range: f32 = Some(light.range * 1.1)
      .filter(|range| !range.is_subnormal())
      .unwrap_or(0.0);
    let range_squared: f32 = range * range;
    let length_squared: f32 = light.direction.x * light.direction.x
      + light.direction.y * light.direction.y
      + light.direction.z * light.direction.z;
    let direction: Vector3d = if length_squared > f32::MIN_POSITIVE {
      let scale: f32 = (1.0 / length_squared).sqrt();

      Vector3d::new(
        light.direction.x * scale,
        light.direction.y * scale,
        light.direction.z * scale,
      )
    } else {
      light.direction
    };

    LevelLight {
      kind: if light.kind == LevelDynamicLight::KIND_DIRECTIONAL {
        LevelLight::KIND_DIRECTIONAL
      } else {
        LevelLight::KIND_POINT
      },
      level: 0,
      diffuse,
      position: light.position,
      direction,
      range,
      range_squared,
      falloff: 1.0
        / (range * (light.attenuation_0 + light.attenuation_1 * range + light.attenuation_2 * range_squared)),
      attenuation_constant: light.attenuation_0,
      attenuation_linear: light.attenuation_1,
      attenuation_quadratic: light.attenuation_2,
      energy,
      triangle: [Vector3d::ZERO; 3],
    }
  }
}
