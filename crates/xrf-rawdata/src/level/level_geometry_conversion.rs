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
  LevelFile, LevelGeomSource, LevelSectorComposition, LevelShadersChunk, LevelVisual, LevelVisualsChunk,
};
use xrf_math::Vector3d;

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_surface_inference::LevelSurfaceInference;
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// A level's static geometry, from the compiled sectors' visuals back into objects the scene places, and the sectors
/// and portals that cull them (decision 11's shells): one object per compiled sector holding every static triangle the
/// sector draws, placed by a scene object and named by the sector, whose index the Build gives back since it numbers
/// sectors in the order the scene lists them (`SceneBuilder::RenumerateSectors`, `BuilderCore.cpp:75`).
pub struct LevelGeometryConversion;

/// One object the decompile writes, by the path the scene references it at.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelDecompiledObject {
  /// Its path under `$objects$` without the extension, `levels\<level>\<level>_sector_<N>`: what the scene object's
  /// `reference_name` names.
  pub reference: String,
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

  /// What names the scene's sectors and portals, as the editor names new ones.
  const SECTOR_PREFIX: &'static str = "sector";
  const PORTAL_PREFIX: &'static str = "portal";

  /// The geometry of the compiled level `level` named `name`, its visuals `visuals` over `source`: each sector's static
  /// visuals (trees and impostors are left for their own objects) decompiled into one object of the sector's name
  /// (`ObjectDecompiler::decompile_level_geometry`), its points in the level's space and its scene object at the origin;
  /// a sector per compiled sector naming its object, and a portal per compiled portal. With `surfaces`, each visual's
  /// triangles are split by the game material they lie on and dressed (decision 14); without, every surface takes the
  /// editor's defaults.
  ///
  /// A sector drawing no static geometry has no object to name, and the editor drops a sector of no item, which would
  /// renumber every sector after it. Where one does, the shells come back without sectors or portals, each such sector
  /// refused, and the Build puts every face in its default sector.
  ///
  /// # Errors
  ///
  /// Returns an error when the level holds no shader table, a sector's visual cannot be read, a decompile refuses its
  /// geometry, or a portal names a sector the level does not hold or is no polygon.
  pub fn decompile<T: ByteOrder, D: ChunkDataSource>(
    name: &str,
    level: &LevelFile,
    visuals: &LevelVisualsChunk,
    source: &LevelGeomSource<D>,
    mut surfaces: Option<&mut LevelSurfaceInference>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<LevelGeometryDecompile> {
    let shaders: &LevelShadersChunk = level
      .shaders
      .as_ref()
      .ok_or_else(|| XrfError::new_invalid_error("The level holds visuals but no shader table"))?;
    let shells: Vec<(SectorGeometry, Option<ObjectDecompile>)> = level
      .get_sectors()
      .iter()
      .map(|sector| {
        let mut read: SectorGeometry = Self::read_sector::<T, D>(visuals, sector.root, shaders, source)?;

        if let Some(surfaces) = surfaces.as_deref_mut() {
          read.geometry = std::mem::take(&mut read.geometry)
            .into_iter()
            .flat_map(|geometry| surfaces.dress(geometry))
            .collect();
        }

        let decompile: Option<ObjectDecompile> = if read.geometry.is_empty() {
          None
        } else {
          Some(ObjectDecompiler::decompile_level_geometry(&read.geometry)?)
        };

        Ok((read, decompile))
      })
      .collect::<XrfResult<_>>()?;
    let mut census: LevelGeometryCensus = LevelGeometryCensus::default();
    let mut objects: Vec<LevelDecompiledObject> = Vec::new();
    let mut scene_objects: Vec<EditableSceneObject> = Vec::new();
    let mut sectors: Vec<EditableSectorObject> = Vec::new();
    let mut sector_names: Vec<String> = Vec::with_capacity(shells.len());
    let mut refused: Vec<LevelSceneRefusal> = Vec::new();

    for (index, (read, decompile)) in shells.into_iter().enumerate() {
      let sector_name: String = context.names.get_next(Self::SECTOR_PREFIX);
      let triangles: usize = read.geometry.iter().map(|it| it.indices.len() / 3).sum();

      census.static_triangles += triangles;
      census.tree_triangles += read.tree_triangles;
      census.impostors += read.impostors;

      let Some(decompile) = decompile else {
        census.empty_sectors += 1;
        refused.push(LevelSceneRefusal {
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
      let (faces, two_sided) = Self::count_faces(&decompile.object);

      census.faces += faces;
      census.two_sided_faces += two_sided;
      // A two-sided face stands for a triangle and its mirror; what neither a face nor a fold accounts for was a repeat.
      census.repeated_triangles += triangles.saturating_sub(faces + two_sided);

      let reference: String = format!("levels\\{name}\\{name}_sector_{index}");
      let mesh: String = Self::get_mesh(&decompile.object).name.clone();

      context.names.reserve(&reference);
      // At the level's origin, its points kept in the level's space: a shell spans its sector, so moving its pivot to
      // its bounds would round the low bits off every point nearer the origin than the pivot, and two sectors would no
      // longer meet exactly where the level had them meet.
      scene_objects.push(EditableSceneObject {
        object: EditableCustomObject::new(reference.clone(), EditableObjectTransform::default()),
        reference: reference.clone(),
        flags: 0,
      });
      sectors.push(EditableSectorObject {
        object: EditableCustomObject::new(sector_name.clone(), EditableObjectTransform::default()),
        color: Self::SECTOR_COLOR,
        is_default: false,
        items: vec![EditableSectorItem {
          object: reference.clone(),
          mesh,
        }],
        map_index: EditableSectorObject::NO_MAP,
      });
      sector_names.push(sector_name);
      objects.push(LevelDecompiledObject { reference, decompile });
    }

    let portals: Vec<EditablePortalObject> = level
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
      .collect::<XrfResult<_>>()?;
    let (sectors, portals) = if refused.is_empty() {
      (sectors, portals)
    } else {
      (Vec::new(), Vec::new())
    };

    Ok(LevelGeometryDecompile {
      objects,
      scene_objects: EditableObjectPart::new(context.guid, scene_objects),
      sectors: EditableObjectPart::new(context.guid, sectors),
      portals: EditableObjectPart::new(context.guid, portals),
      census,
      refused,
    })
  }

  /// The static geometry a sector's root reaches: every drawable visual but trees and their impostors, which are
  /// counted.
  fn read_sector<T: ByteOrder, D: ChunkDataSource>(
    visuals: &LevelVisualsChunk,
    root: u32,
    shaders: &LevelShadersChunk,
    source: &LevelGeomSource<D>,
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
      } else {
        read.geometry.push(geometry);
      }
    }

    Ok(read)
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
