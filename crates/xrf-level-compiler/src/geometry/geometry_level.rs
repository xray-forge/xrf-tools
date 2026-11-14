use xrf_error::{XrfError, XrfResult};
use xrf_geometry::ProgressiveWindow;
use xrf_level::{
  BuildCformFile, BuildPrjFile, LevelBundle, LevelCformFile, LevelCformGeometry, LevelCformHeader, LevelDynamicLight,
  LevelGeomBuffers, LevelGeomPacker, LevelGeomSlideWindowItem, LevelGlow, LevelHeaderChunk, LevelLight,
  LevelLightsChunk, LevelLightsFile, LevelPortal, LevelSector, LevelShaderEntry, LevelShadersChunk,
  LevelVertexQuantizer, LevelVisualRecord,
};
use xrf_math::{Sphere, Vector3d};
use xrf_ogf::{
  OgfFastpathChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfLodDefinitionChunk, OgfLodFacet, OgfLodVertex,
  OgfModelType, OgfSlideWindow, OgfSwiContainerChunk, OgfSwiDataChunk, OgfTreeColor, OgfTreeDefinitionChunk,
};
use xrf_shaders::ShaderCompilerFile;

use crate::compiler_shaders::CompilerShaders;
use crate::compiler_texture::CompilerTexture;
use crate::geometry::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::geometry_bounds::GeometryBounds;
use crate::geometry::geometry_collision::GeometryCollision;
use crate::geometry::geometry_draft_light::GeometryDraftLight;
use crate::geometry::geometry_hierarchy::GeometryHierarchy;
use crate::geometry::geometry_level_census::GeometryLevelCensus;
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

/// Everything xrLC writes for a level's geometry, compiled from its `build.prj` under draft light: the `level` file,
/// its render geometry, the game's collision form, the compilers' ray model and lights, and the draft lightmaps.
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
  /// Whether a visual draws from the draft lightmap pair, which is then written.
  pub has_lightmaps: bool,
  /// What the compile made, for its report.
  pub census: GeometryLevelCensus,
}

impl GeometryLevel {
  /// `XRCL_PRODUCTION_VERSION`, the version the engine asserts a level carries.
  pub const VERSION: u16 = 14;

  /// The level of `project`, its compiler shaders from `library` and its textures as `textures` loads them.
  ///
  /// # Errors
  ///
  /// Returns an error where a phase refuses the project, and for a static light layer `build.prj` lacks.
  pub fn compile(
    project: &BuildPrjFile,
    library: &ShaderCompilerFile,
    textures: &[CompilerTexture],
  ) -> XrfResult<Self> {
    let shaders: CompilerShaders = CompilerShaders::resolve(project, library)?;
    let mut mesh: GeometryMesh = GeometryMesh::load(project)?;

    mesh.optimize(project.params.weld_distance)?;
    mesh.tessellate();

    let normals: GeometryNormals = GeometryNormals::of(&mesh);
    let collision: LevelCformGeometry = GeometryCollision::build(&mesh, project, &shaders)?;
    let coordinates: GeometryBaseCoordinates = GeometryBaseCoordinates::of(&mesh, &normals);
    let tangents: GeometryTangents = GeometryTangents::of(&mesh, &normals, &coordinates);
    let ray_model: BuildCformFile = GeometryRayModel::build(&mesh, &normals, &coordinates, project, &shaders)?;
    let splits: GeometrySplits = GeometrySplits::of(&mesh, &shaders);
    let visuals: GeometryVisuals = GeometryVisuals::build(&GeometryVisualsRequest {
      mesh: &mesh,
      normals: &normals,
      coordinates: &coordinates,
      tangents: &tangents,
      splits: &splits,
      project,
      shaders: &shaders,
      textures,
    })?;
    let models: GeometryModels = GeometryModels::build(project, &shaders, textures)?;
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

    let has_lightmaps: bool = visuals.visuals.iter().any(|visual| {
      visual
        .textures
        .iter()
        .any(|name| name == GeometryDraftLight::LIGHTMAP_TEXTURES[0])
    });
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
      lights: Self::get_lights(project)?,
      has_lightmaps,
      census,
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

  /// The draft lightmap pair's texels: `_1` the static light (none) and the sun in alpha, `_2` the sun in every colour
  /// and the hemisphere in alpha (`lm_layer::Pack`, `Pack_hemi`).
  pub fn get_lightmap_texels() -> [[u8; 4]; 2] {
    let term = |value: f32| LevelVertexQuantizer::get_light_term(value);
    let sun: u8 = term(GeometryDraftLight::SUN);
    let rgb: u8 = term(GeometryDraftLight::STATIC_LIGHT);

    [
      [rgb, rgb, rgb, sun],
      [sun, sun, sun, term(GeometryDraftLight::HEMISPHERE)],
    ]
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
  /// (`Build_Load.cpp:211`), chunks 0, 1 and 2.
  fn get_lights(project: &BuildPrjFile) -> XrfResult<LevelLightsFile> {
    let layer = |name: &str| -> XrfResult<Vec<LevelLight>> {
      let control: usize = project
        .light_controls
        .iter()
        .position(|control| control.name.value.eq_ignore_ascii_case(name))
        .ok_or_else(|| XrfError::new_invalid_error(format!("build.prj has no {name} light layer")))?;
      let lights: Vec<LevelLight> = project
        .static_lights
        .iter()
        .filter(|light| light.controller as usize == control)
        .map(Self::get_light)
        .collect();

      if lights.is_empty() {
        return Err(XrfError::new_invalid_error(format!(
          "build.prj's {name} light layer holds no light"
        )));
      }

      Ok(lights)
    };

    Ok(LevelLightsFile {
      chunks: vec![
        LevelLightsChunk::Lights {
          id: 0,
          lights: layer("$static")?,
        },
        LevelLightsChunk::Lights {
          id: 1,
          lights: layer("$hemi")?,
        },
        LevelLightsChunk::Lights {
          id: 2,
          lights: layer("$sun")?,
        },
      ],
    })
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
