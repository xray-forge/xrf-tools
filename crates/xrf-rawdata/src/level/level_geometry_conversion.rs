use std::collections::BTreeMap;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::ChunkDataSource;
use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditableObjectTransform, EditablePortalObject, EditableSceneObject,
  EditableSectorItem, EditableSectorObject,
};
use xrf_editable_object::{EditableMesh, EditableObjectFile};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelCformFile, LevelFile, LevelGeomFile, LevelGeomSource, LevelHomFile, LevelSectorComposition, LevelShadersChunk,
  LevelSomFile, LevelVisual, LevelVisualsChunk,
};
use xrf_math::{Matrix4x4, Vector3d};

use crate::level::level_collision_index::LevelCollisionIndex;
use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_occluder_conversion::LevelOccluderConversion;
use crate::level::level_surface_inference::LevelSurfaceInference;
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// A level's geometry, from the compiled level back into objects the scene places, and the sectors and portals that
/// cull them (decision 11's shells): one object per compiled sector holding every static triangle the sector draws,
/// named by the sector, whose index the Build gives back since it numbers sectors in the order the scene lists them
/// (`SceneBuilder::RenumerateSectors`, `BuilderCore.cpp:75`); the collision no render triangle draws; and the occluders.
///
/// Every object keeps the level's coordinates and stands at the origin: a shell spans its sector, so a pivot inside it
/// would round the low bits off every point nearer the origin than the pivot, and two sectors would no longer meet
/// exactly where the level had them meet (decision 11, as amended).
pub struct LevelGeometryConversion;

/// What a level's geometry decompiles from: the compiled bundle named `name`, its visuals and the buffers they draw
/// from, and where the level's directory holds them, its collision form, indexed, and its occlusion meshes.
pub struct LevelGeometrySources<'a, D: ChunkDataSource> {
  pub name: &'a str,
  pub level: &'a LevelFile,
  pub visuals: &'a LevelVisualsChunk,
  pub geom: &'a LevelGeomSource<D>,
  pub collision: Option<&'a mut LevelCollisionIndex>,
  pub hom: Option<&'a LevelHomFile>,
  pub som: Option<&'a LevelSomFile>,
}

/// One object the decompile writes, by the path the scene references it at.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelDecompiledObject {
  /// Its path under `$objects$` without the extension, such as `levels\<level>\<level>_sector_<N>`: what the scene
  /// object's `reference_name` names.
  pub reference: String,
  /// The compiled file it came from, by its name in the level's directory, which its sidecar records: `level.geom`
  /// for a sector's shell, `level.cform` for the collision object, `level.hom` and `level.som` for the occluders.
  pub source: &'static str,
  pub decompile: ObjectDecompile,
}

/// What the static geometry's decompile counts, so every triangle the compiled level draws is either in a written
/// object or accounted for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeometryCensus {
  /// Full-detail triangles of the sectors' static visuals, read.
  pub static_triangles: usize,
  /// Faces the written objects hold.
  pub faces: usize,
  /// Faces of those that are two-sided, each folded from a triangle and its mirror.
  pub two_sided_faces: usize,
  /// Triangles dropped as an exact repeat of another of their surface.
  pub repeated_triangles: usize,
  /// Full-detail triangles of tree instances, left for their own objects.
  pub tree_triangles: usize,
  /// Impostors standing for trees afar, which the build makes again from the trees.
  pub impostors: usize,
  /// Sectors drawing no static geometry, such as one of trees alone, which no shell stands for.
  pub empty_sectors: usize,
  /// Faces of the collision form, where the level's directory held one.
  pub collision_faces: usize,
  /// Collision faces only a tree's triangles lie on: trunks, which come back with their trees.
  pub tree_collision_faces: usize,
  /// Collision faces no render triangle lies on, which the collision object holds.
  pub collision_only_faces: usize,
  /// Polygons of `level.hom` and `level.som`, which `<level>_hom` and `<level>_som` hold.
  pub occluder_polygons: usize,
  pub sound_occluder_polygons: usize,
  /// Sound occlusion factors no game material of the library has, whose polygons take `default`.
  pub unmatched_sound_occlusion_factors: usize,
}

/// A level's geometry decompiled: the objects to write, and the parts placing them and culling them.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelGeometryDecompile {
  pub objects: Vec<LevelDecompiledObject>,
  pub scene_objects: EditableObjectPart<EditableSceneObject>,
  /// Empty where a sector draws no static geometry, with the portals: see [`LevelGeometryConversion::decompile`].
  pub sectors: EditableObjectPart<EditableSectorObject>,
  pub portals: EditableObjectPart<EditablePortalObject>,
  pub census: LevelGeometryCensus,
  /// Each sector left out, and why.
  pub refused: Vec<LevelSceneRefusal>,
}

impl LevelGeometryDecompile {
  /// Adds `object` with a scene object placing it at the origin under its reference, which `context` keeps from every
  /// name generated after.
  fn place(&mut self, object: LevelDecompiledObject, context: &mut LevelDecompileContext) {
    context.names.reserve(&object.reference);
    self.scene_objects.objects.push(EditableSceneObject {
      object: EditableCustomObject::new(object.reference.clone(), EditableObjectTransform::default()),
      reference: object.reference.clone(),
      flags: 0,
    });
    self.objects.push(object);
  }
}

/// One mesh of the collision object: the sector its faces lie in, which names it, and how many faces it holds.
struct CollisionMesh {
  sector: u16,
  name: String,
  faces: usize,
}

/// One sector's static geometry read, and what it left out.
struct SectorGeometry {
  geometry: Vec<LevelVisualGeometry>,
  tree_triangles: usize,
  impostors: usize,
}

impl LevelGeometryConversion {
  /// A new sector's colour, white and clear (`CSector::Construct`, `sector.cpp:55`), which the compiled level does not
  /// keep.
  pub const SECTOR_COLOR: [u8; 4] = [255, 255, 255, 0];

  /// How GSC dressed the collision-only props of its own scenes (`prop\prop_fake_kollision` in the SDK's objects), which
  /// the collision object takes: the build reads only the compiler shader, of collision and no rendering.
  pub const COLLISION_TEXTURE: &'static str = "prop\\prop_fake_kollision";
  pub const COLLISION_ENGINE_SHADER: &'static str = "def_shaders\\def_trans_v";
  pub const COLLISION_COMPILER_SHADER: &'static str = "def_shaders\\def_kolizion_vertex";

  /// What the collision object's meshes are named before their sector, `collision_<sector>`.
  const COLLISION_MESH_PREFIX: &'static str = "collision";

  /// What names the scene's sectors and portals, as the editor names new ones.
  const SECTOR_PREFIX: &'static str = "sector";
  const PORTAL_PREFIX: &'static str = "portal";

  /// The geometry of a compiled level, every object of it at the origin in the level's space, placed by a scene object:
  ///
  /// - each sector's static visuals (trees and impostors are left for their own objects) decompiled into one object of
  ///   the sector's name (`ObjectDecompiler::decompile_level_geometry`), a sector per compiled sector naming it, and a
  ///   portal per compiled portal; with `surfaces`, each visual's triangles are split by the game material they lie on
  ///   and dressed (decision 14), else every surface takes the editor's defaults;
  /// - with the collision form, the faces no render triangle lies on, as `<level>_collision`;
  /// - `level.hom` and `level.som`, as `<level>_hom` and `<level>_som` ([`LevelOccluderConversion`]).
  ///
  /// A sector drawing no static geometry has no object to name, and the editor drops a sector of no item, which would
  /// renumber every sector after it. Where one does, the objects come back without sectors or portals, each such sector
  /// refused, and the Build puts every face in its default sector.
  ///
  /// # Errors
  ///
  /// Returns an error when the level holds no shader table, a sector's visual cannot be read, a decompile refuses its
  /// geometry, or a portal names a sector the level does not hold or is no polygon.
  pub fn decompile<T: ByteOrder, D: ChunkDataSource>(
    mut sources: LevelGeometrySources<'_, D>,
    mut surfaces: Option<&mut LevelSurfaceInference>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<LevelGeometryDecompile> {
    let name: &str = sources.name;
    let mut decompile: LevelGeometryDecompile = LevelGeometryDecompile {
      objects: Vec::new(),
      scene_objects: EditableObjectPart::new(context.guid, Vec::new()),
      sectors: EditableObjectPart::new(context.guid, Vec::new()),
      portals: EditableObjectPart::new(context.guid, Vec::new()),
      census: LevelGeometryCensus::default(),
      refused: Vec::new(),
    };
    let sector_names: Vec<String> =
      Self::decompile_shells::<T, D>(&mut decompile, &mut sources, surfaces.as_deref_mut(), context)?;

    if let Some(collision) = sources.collision.as_deref() {
      Self::decompile_collision(&mut decompile, name, collision, surfaces.as_deref_mut(), context)?;
    }

    if let Some(hom) = sources.hom {
      decompile.census.occluder_polygons = hom.polygons.len();

      if let Some(object) = LevelOccluderConversion::decompile_hom(name, hom)? {
        decompile.place(object, context);
      }
    }

    if let Some(som) = sources.som {
      let (object, census) = LevelOccluderConversion::decompile_som(
        name,
        som,
        surfaces.as_deref().map(LevelSurfaceInference::get_game_materials),
      )?;

      decompile.census.sound_occluder_polygons = census.polygons;
      decompile.census.unmatched_sound_occlusion_factors = census.unmatched_factors.len();

      if let Some(object) = object {
        decompile.place(object, context);
      }
    }

    // With a sector left out, the rest would renumber, so none is written, and no portal joining them.
    if decompile.refused.is_empty() {
      decompile.portals.objects = Self::decompile_portals(sources.level, &sector_names, context)?;
    } else {
      decompile.sectors.objects.clear();
    }

    Ok(decompile)
  }

  /// Each sector's shell and the sector naming it, into `decompile`; the sectors' names by compiled index, an empty
  /// sector's too, which the portals join by.
  fn decompile_shells<T: ByteOrder, D: ChunkDataSource>(
    decompile: &mut LevelGeometryDecompile,
    sources: &mut LevelGeometrySources<'_, D>,
    mut surfaces: Option<&mut LevelSurfaceInference>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<Vec<String>> {
    let (name, level, visuals, source) = (sources.name, sources.level, sources.visuals, sources.geom);
    let mut collision: Option<&mut LevelCollisionIndex> = sources.collision.as_deref_mut();
    let shaders: &LevelShadersChunk = level
      .shaders
      .as_ref()
      .ok_or_else(|| XrfError::new_invalid_error("The level holds visuals but no shader table"))?;
    let shells: Vec<(SectorGeometry, Option<ObjectDecompile>)> = level
      .get_sectors()
      .iter()
      .map(|sector| {
        let mut read: SectorGeometry =
          Self::read_sector::<T, D>(visuals, sector.root, shaders, source, collision.as_deref_mut())?;

        match (collision.as_deref_mut(), surfaces.as_deref_mut()) {
          (Some(collision), Some(surfaces)) => {
            read.geometry = std::mem::take(&mut read.geometry)
              .into_iter()
              .flat_map(|geometry| surfaces.dress(collision, geometry))
              .collect();
          }
          (Some(collision), None) => {
            for geometry in &read.geometry {
              for points in Self::list_triangles(geometry, None) {
                collision.find_static(&points);
              }
            }
          }
          (None, _) => {}
        }

        let decompile: Option<ObjectDecompile> = if read.geometry.is_empty() {
          None
        } else {
          Some(ObjectDecompiler::decompile_level_geometry(&read.geometry)?)
        };

        Ok((read, decompile))
      })
      .collect::<XrfResult<_>>()?;
    let mut sector_names: Vec<String> = Vec::with_capacity(shells.len());

    for (index, (read, shell)) in shells.into_iter().enumerate() {
      let sector_name: String = context.names.get_next(Self::SECTOR_PREFIX);
      let triangles: usize = read.geometry.iter().map(|it| it.indices.len() / 3).sum();
      let census: &mut LevelGeometryCensus = &mut decompile.census;

      census.static_triangles += triangles;
      census.tree_triangles += read.tree_triangles;
      census.impostors += read.impostors;

      let Some(shell) = shell else {
        census.empty_sectors += 1;
        decompile.refused.push(LevelSceneRefusal {
          file: String::from(LevelFile::FILE_NAME),
          object: format!("sector {index}"),
          reason: String::from(
            "The sector draws no static geometry, so no object would name it, and the scene leaves out every sector \
             and portal rather than renumber them",
          ),
        });
        sector_names.push(sector_name);
        continue;
      };
      let (faces, two_sided) = Self::count_faces(&shell.object);

      census.faces += faces;
      census.two_sided_faces += two_sided;
      // A two-sided face stands for a triangle and its mirror; what neither a face nor a fold accounts for was a repeat.
      census.repeated_triangles += triangles.saturating_sub(faces + two_sided);

      let reference: String = format!("levels\\{name}\\{name}_sector_{index}");

      decompile.sectors.objects.push(EditableSectorObject {
        object: EditableCustomObject::new(sector_name.clone(), EditableObjectTransform::default()),
        color: Self::SECTOR_COLOR,
        is_default: false,
        items: vec![EditableSectorItem {
          object: reference.clone(),
          mesh: Self::get_mesh(&shell.object).name.clone(),
        }],
        map_index: EditableSectorObject::NO_MAP,
      });
      sector_names.push(sector_name);
      decompile.place(
        LevelDecompiledObject {
          reference,
          source: LevelGeomFile::FILE_NAME,
          decompile: shell,
        },
        context,
      );
    }

    Ok(sector_names)
  }

  /// A portal per compiled portal, joining the sectors `sector_names` names by compiled index.
  fn decompile_portals(
    level: &LevelFile,
    sector_names: &[String],
    context: &mut LevelDecompileContext,
  ) -> XrfResult<Vec<EditablePortalObject>> {
    level
      .get_portals()
      .iter()
      .enumerate()
      .map(|(index, portal)| {
        let sector = |sector: u16| -> XrfResult<String> {
          sector_names.get(usize::from(sector)).cloned().ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Portal {index} joins sector {sector}, past the level's {}",
              sector_names.len()
            ))
          })
        };
        let polygon: &[Vector3d] = portal.get_polygon().ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Portal {index} has {} corners, which is no polygon",
            portal.vertex_count
          ))
        })?;

        Ok(EditablePortalObject {
          object: EditableCustomObject::new(
            context.names.get_next(Self::PORTAL_PREFIX),
            EditableObjectTransform::default(),
          ),
          sector_front: sector(portal.sector_front)?,
          sector_back: sector(portal.sector_back)?,
          vertices: polygon.to_vec(),
        })
      })
      .collect()
  }

  /// The static geometry a sector's root reaches: every drawable visual but trees and their impostors, which are
  /// counted.
  fn read_sector<T: ByteOrder, D: ChunkDataSource>(
    visuals: &LevelVisualsChunk,
    root: u32,
    shaders: &LevelShadersChunk,
    source: &LevelGeomSource<D>,
    mut collision: Option<&mut LevelCollisionIndex>,
  ) -> XrfResult<SectorGeometry> {
    let composition: LevelSectorComposition = LevelSectorComposition::of(visuals, root);
    let mut read: SectorGeometry = SectorGeometry {
      geometry: Vec::new(),
      tree_triangles: 0,
      impostors: 0,
    };

    if let Some(unknown) = composition.unknown.first() {
      return Err(XrfError::new_invalid_error(format!(
        "A sector's root {root} reaches visual {unknown}, past the level's {}",
        visuals.visuals.len()
      )));
    }

    for index in composition.drawables {
      let visual: &LevelVisual = &visuals.visuals[index as usize];

      if visual.lod.is_some() {
        read.impostors += 1;
        continue;
      }

      let Some(geometry) = LevelVisualGeometry::read::<T, D>(visual, shaders, source)
        .map_err(|error| XrfError::new_invalid_error(format!("Visual {index} was not read: {error}")))?
      else {
        continue;
      };

      if visual.tree.is_some() {
        read.tree_triangles += geometry.indices.len() / 3;

        // A trunk's collision faces come back with its tree, not as collision of their own.
        if let Some(collision) = collision.as_deref_mut() {
          for points in Self::list_triangles(&geometry, visual.get_placement()) {
            collision.cover_tree(&points);
          }
        }
      } else {
        read.geometry.push(geometry);
      }
    }

    Ok(read)
  }

  /// The triangles of `geometry`, placed in the level by `placement` where it stands in a space of its own; an index
  /// past its vertices is left out, as the decompile refuses it on its own.
  fn list_triangles<'a>(
    geometry: &'a LevelVisualGeometry,
    placement: Option<&'a Matrix4x4>,
  ) -> impl Iterator<Item = [Vector3d; 3]> + 'a {
    geometry.indices.as_chunks::<3>().0.iter().filter_map(move |triangle| {
      let point = |index: &u16| -> Option<Vector3d> {
        let position: &Vector3d = &geometry.vertices.get(usize::from(*index))?.position;

        Some(placement.map_or(*position, |matrix| matrix.transform_point(position)))
      };

      Some([point(&triangle[0])?, point(&triangle[1])?, point(&triangle[2])?])
    })
  }

  /// The collision faces no render triangle covers, as one object, `<level>_collision`, of a mesh per sector they lie
  /// in, `collision_<sector>`, which that sector names, so the faces keep the sector the renderer finds the camera's
  /// by: dressed as GSC dressed its collision-only props, a compiler shader of collision and no rendering, with each
  /// face's game material where `surfaces` names them. Counted into `decompile` with the faces only trees cover.
  fn decompile_collision(
    decompile: &mut LevelGeometryDecompile,
    name: &str,
    collision: &LevelCollisionIndex,
    surfaces: Option<&mut LevelSurfaceInference>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult {
    decompile.census.collision_faces = collision.count_faces();
    decompile.census.tree_collision_faces = collision.count_tree_covered();

    let Some((object, meshes)) = Self::decompile_collision_object(name, collision, surfaces)? else {
      return Ok(());
    };

    decompile.census.collision_only_faces = meshes.iter().map(|mesh| mesh.faces).sum();

    for mesh in meshes {
      // A sector left out leaves out every sector, so no item is needed.
      if decompile.refused.is_empty()
        && let Some(sector) = decompile.sectors.objects.get_mut(usize::from(mesh.sector))
      {
        sector.items.push(EditableSectorItem {
          object: object.reference.clone(),
          mesh: mesh.name,
        });
      }
    }

    decompile.place(object, context);

    Ok(())
  }

  /// The collision object of the faces no render triangle covers, with each mesh's sector and faces; none where every
  /// face is covered.
  fn decompile_collision_object(
    name: &str,
    collision: &LevelCollisionIndex,
    mut surfaces: Option<&mut LevelSurfaceInference>,
  ) -> XrfResult<Option<(LevelDecompiledObject, Vec<CollisionMesh>)>> {
    let mut by_sector: BTreeMap<u16, BTreeMap<u16, Vec<[Vector3d; 3]>>> = BTreeMap::new();

    for (_, face) in collision.get_uncovered() {
      by_sector
        .entry(face.sector)
        .or_default()
        .entry(face.material)
        .or_default()
        .push(collision.get_triangle(face));
    }

    let mut meshes: Vec<(String, ObjectDecompile)> = Vec::with_capacity(by_sector.len());
    let mut counts: Vec<CollisionMesh> = Vec::with_capacity(by_sector.len());

    for (sector, materials) in by_sector {
      let mut geometry: Vec<LevelVisualGeometry> = Vec::new();

      for (material, triangles) in materials {
        let dress: SurfaceDress = SurfaceDress {
          compiler_shader: String::from(Self::COLLISION_COMPILER_SHADER),
          game_material: surfaces.as_deref_mut().map_or_else(
            || String::from(SurfaceDress::DEFAULT_SETTING),
            |surfaces| surfaces.get_material_name(Some(material)),
          ),
        };

        geometry.extend(LevelVisualGeometry::of_triangles(
          Self::COLLISION_TEXTURE,
          Self::COLLISION_ENGINE_SHADER,
          &dress,
          &triangles,
        ));
      }

      let faces: usize = geometry.iter().map(|it| it.indices.len() / 3).sum();
      let mesh: String = format!("{}_{sector}", Self::COLLISION_MESH_PREFIX);

      meshes.push((mesh.clone(), ObjectDecompiler::decompile_level_geometry(&geometry)?));
      counts.push(CollisionMesh {
        sector,
        name: mesh,
        faces,
      });
    }

    Ok(ObjectDecompile::merge_meshes(meshes).map(|decompile| {
      (
        LevelDecompiledObject {
          reference: format!("levels\\{name}\\{name}_collision"),
          source: LevelCformFile::FILE_NAME,
          decompile,
        },
        counts,
      )
    }))
  }

  /// The one mesh a decompiled object holds.
  fn get_mesh(object: &EditableObjectFile) -> &EditableMesh {
    object
      .meshes
      .as_deref()
      .and_then(<[EditableMesh]>::first)
      .expect("a decompiled object holds one mesh")
  }

  /// The faces `object`'s mesh holds, and how many of them are two-sided.
  fn count_faces(object: &EditableObjectFile) -> (usize, usize) {
    let mesh: &EditableMesh = Self::get_mesh(object);
    let surfaces = object.surfaces.as_deref().unwrap_or_default();
    let two_sided: usize = mesh
      .surface_faces
      .iter()
      .filter(|faces| {
        surfaces
          .iter()
          .any(|surface| surface.name == faces.surface && surface.is_two_sided())
      })
      .map(|faces| faces.faces.len())
      .sum();

    (mesh.faces.len(), two_sided)
  }
}
