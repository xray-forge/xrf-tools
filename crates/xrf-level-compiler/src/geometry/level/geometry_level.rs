use xrf_error::{XrfError, XrfResult};
use xrf_geometry::ProgressiveWindow;
use xrf_level::{
  BuildCformFile, BuildPrjFile, LevelBundle, LevelCformFile, LevelCformGeometry, LevelCformHeader, LevelGeomBuffers,
  LevelGeomPacker, LevelGeomSlideWindowItem, LevelGlow, LevelHeaderChunk, LevelLightsFile, LevelPortal, LevelSector,
  LevelShaderEntry, LevelShadersChunk, LevelVisualRecord,
};
use xrf_math::Sphere;
use xrf_ogf::{
  OgfFastpathChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfLodDefinitionChunk, OgfLodFacet, OgfLodVertex,
  OgfModelType, OgfSlideWindow, OgfSwiContainerChunk, OgfSwiDataChunk, OgfTreeColor, OgfTreeDefinitionChunk,
};

use crate::geometry::collision::geometry_collision::GeometryCollision;
use crate::geometry::collision::geometry_ray_model::GeometryRayModel;
use crate::geometry::hierarchy::geometry_bounds::GeometryBounds;
use crate::geometry::hierarchy::geometry_hierarchy::GeometryHierarchy;
use crate::geometry::hierarchy::geometry_node_kind::GeometryNodeKind;
use crate::geometry::level::geometry_level_census::GeometryLevelCensus;
use crate::geometry::level::geometry_level_request::GeometryLevelRequest;
use crate::geometry::level::geometry_packed_visual::GeometryPackedVisual;
use crate::geometry::level::geometry_shader_table::GeometryShaderTable;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::model::geometry_models::GeometryModels;
use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::geometry::surface::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::geometry::surface::geometry_tangents::GeometryTangents;
use crate::geometry::visual::geometry_visual::GeometryVisual;
use crate::geometry::visual::geometry_visuals::GeometryVisuals;
use crate::geometry::visual::geometry_visuals_request::GeometryVisualsRequest;
use crate::input::compiler_shaders::CompilerShaders;
use crate::input::compiler_texture::CompilerTexture;
use crate::light::bake::light_bake::LightBake;
use crate::light::bake::light_bake_input::LightBakeInput;
use crate::light::bake::light_bake_result::LightBakeResult;
use crate::light::bake::light_baked::LightBaked;
use crate::light::bake::light_baker::LightBaker;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::light_draft::LightDraft;
use crate::light::lightmap::atlas::lightmap_images::LightmapImages;
use crate::light::source::light_layers::LightLayers;

/// Everything xrLC writes for a level's geometry, compiled from its `build.prj`, lit by the light baker or with neutral
/// light: the `level` file, its render geometry, the game's collision form, the compilers' ray model and lights, the
/// lightmap pages and the implicit maps.
#[derive(Debug)]
pub(crate) struct GeometryLevel {
  pub(crate) bundle: LevelBundle,
  /// `level.geom`.
  pub(crate) geometry: LevelGeomBuffers,
  /// `level.geomx`.
  pub(crate) fast: LevelGeomBuffers,
  /// `level.cform`.
  pub(crate) collision: LevelCformGeometry,
  /// `build.cform`.
  pub(crate) ray_model: BuildCformFile,
  /// `build.lights`.
  pub(crate) lights: LevelLightsFile,
  /// Each lightmap page's two textures: the baked pages
  /// ([`LightmapAtlas::compose_all`](crate::light::lightmap::atlas::lightmap_atlas::LightmapAtlas::compose_all)), or
  /// the one pair of neutral light ([`LightDraft::get_lightmaps`]).
  pub(crate) lightmaps: LightmapImages,
  /// Each texture lit implicitly, such as terrain's; none unlit or at draft build quality.
  pub(crate) implicit_maps: Vec<ImplicitMap>,
  /// What the compile made, for its report.
  pub(crate) census: GeometryLevelCensus,
  /// How the light was baked, for the light step's report; none for the neutral light.
  pub(crate) light: Option<LightBake>,
}

impl GeometryLevel {
  /// `XRCL_PRODUCTION_VERSION`, the version the engine asserts a level carries.
  pub(crate) const VERSION: u16 = 14;

  /// The level the request's project compiles to, its compiler shaders from its library and its textures as it loads
  /// them: xrLC's geometry phases, then, where the request asks for light, the light baked at its quality
  /// ([`LightBaker::bake`]), its pages deciding which splits join into a visual; else the neutral light
  /// ([`LightDraft`]), every lightmapped visual on its one pair.
  ///
  /// # Errors
  ///
  /// Returns an error where a phase refuses the project, for a static light layer `build.prj` lacks, and where the bake
  /// fails ([`LightBaker::bake`]).
  pub(crate) fn compile(request: &GeometryLevelRequest<'_>) -> XrfResult<Self> {
    let GeometryLevelRequest {
      project,
      library,
      textures,
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
    let lights: LevelLightsFile = LightLayers::build(project, quality)?;
    let (mut splits, light): (GeometrySplits, Option<LightBakeResult>) = match request.light {
      Some(tracing) => {
        let LightBaked { splits, light } = LightBaker::bake(&LightBakeInput {
          level: request,
          tracing,
          mesh: &mesh,
          normals: &normals,
          coordinates: &coordinates,
          shaders: &shaders,
          ray_model: &ray_model,
          lights: &lights,
        })?;

        (splits, Some(light))
      }
      None => (GeometrySplits::cut(&mesh, &shaders, None).splits, None),
    };

    // Splits on one lightmap page only are joined, so the light decides the visuals.
    splits.merge(&mesh, light.as_ref().map(|light| light.faces.pages.as_slice()));

    let visuals: GeometryVisuals = GeometryVisuals::build(&GeometryVisualsRequest {
      mesh: &mesh,
      normals: &normals,
      coordinates: &coordinates,
      tangents: &tangents,
      splits: &splits,
      project,
      shaders: &shaders,
      textures,
      lights: light.as_ref().map(|light| light.vertices.as_slice()),
      lightmaps: light.as_ref().map(|light| &light.faces),
    })?;
    let models: GeometryModels =
      GeometryModels::build(project, &shaders, textures, light.as_ref().map(|light| &light.models))?;
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

    // An empty first entry, then the glows', then the visuals' as they are saved (`xrLC/Build.cpp`).
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
            geometry: Some(Self::get_container(placed, visual.vertices.len(), visual.faces.len())?),
            swi: visual.windows.as_deref().map(Self::get_windows),
            fastpath: placed
              .fast_vertices
              .zip(placed.fast_indices)
              .map(|(vertices, indices)| -> XrfResult<OgfFastpathChunk> {
                Ok(OgfFastpathChunk {
                  geometry: OgfGeometryContainerChunk {
                    vertex_buffer_id: vertices.buffer,
                    vertex_base: vertices.start,
                    vertex_count: Self::get_count(visual.fast_vertices.len(), "fast path vertices")?,
                    index_buffer_id: indices.buffer,
                    index_base: indices.start,
                    index_count: Self::get_count(visual.fast_faces.len() * 3, "fast path indices")?,
                  },
                  swi: visual.fast_windows.as_deref().map(Self::get_windows),
                })
              })
              .transpose()?,
            tree: None,
            swi_container: None,
            children: None,
            lod: None,
          }
        }
        GeometryNodeKind::Tree(index) => {
          let tree = &models.trees[*index as usize];
          let visual = &models.models[tree.model as usize].visuals[tree.visual as usize];

          let placed: &GeometryPackedVisual = &packed_models[tree.model as usize][tree.visual as usize];
          let model_type: OgfModelType = if placed.slide_window.is_some() {
            OgfModelType::TreePm
          } else {
            OgfModelType::TreeSt
          };

          LevelVisualRecord {
            header: header(model_type, table.register(&visual.shader)),
            texture: None,
            geometry: Some(Self::get_container(placed, visual.vertices.len(), visual.faces.len())?),
            swi: None,
            fastpath: None,
            tree: Some(OgfTreeDefinitionChunk {
              transform: tree.transform,
              scale: OgfTreeColor::from(tree.scale),
              bias: OgfTreeColor::from(tree.bias),
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
            children: Some(
              (Self::get_count(first_tree, "visuals")?..Self::get_count(first_tree + lod.trees.len(), "visuals")?)
                .collect(),
            ),
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
    };
    let (lightmaps, implicit_maps, light) = match light {
      Some(light) => (light.lightmaps, light.implicit_maps, Some(light.bake)),
      None => (LightDraft::get_lightmaps(), Vec::new(), None),
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
      light,
    })
  }

  /// The `level.cform` file of the collision form.
  ///
  /// # Errors
  ///
  /// Returns an error for a form of no vertex.
  pub(crate) fn get_collision_file(&self) -> XrfResult<LevelCformFile> {
    Ok(LevelCformFile {
      header: LevelCformHeader::of(&self.collision)?,
    })
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
  ///
  /// # Errors
  ///
  /// Returns an error for a count past the file's 32 bits.
  fn get_container(
    placed: &GeometryPackedVisual,
    vertices: usize,
    faces: usize,
  ) -> XrfResult<OgfGeometryContainerChunk> {
    Ok(OgfGeometryContainerChunk {
      vertex_buffer_id: placed.vertices.buffer,
      vertex_base: placed.vertices.start,
      vertex_count: Self::get_count(vertices, "vertices")?,
      index_buffer_id: placed.indices.buffer,
      index_base: placed.indices.start,
      index_count: Self::get_count(faces * 3, "indices")?,
    })
  }

  /// `count` of `what` as the level's files write it, refused past their 32 bits.
  fn get_count(count: usize, what: &str) -> XrfResult<u32> {
    u32::try_from(count)
      .map_err(|_| XrfError::new_invalid_error(format!("{count} {what} pass a level's 32-bit counts")))
  }
}
