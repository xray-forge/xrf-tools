use std::collections::{BTreeMap, HashMap};

use xrf_editable_level::EditableObjectTransform;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelVisual};
use xrf_math::{Aabb, Matrix4x4, Vector3d};
use xrf_ogf::OgfLodDefinitionChunk;

use crate::level::level_collision_index::LevelCollisionIndex;
use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_geometry_conversion::LevelDecompiledObject;
use crate::level::level_lod_strip::LevelLodStrip;
use crate::level::level_surface_inference::LevelSurfaceInference;
use crate::level::level_tree_impostor::{LevelTreeImpostor, LevelTreeImpostorAtlas};
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// A level's trees folded back into the multi-use objects the Build placed them from (`SceneBuilder::BuildMUObject`).
/// xrLC drew each placement as one tree visual (`MT_TREE_ST`, `MT_TREE_PM`) per subdivision of its model, in the model's
/// own space with the placement's matrix (`OGF_TREEDEF2`), under one `MT_LOD` where the model had an impostor
/// (`xrMU_Model_export_OGF.cpp`). So a placement is the visuals one `MT_LOD` parents, or that share a matrix where none
/// does, and a model is the subdivisions every placement of it names: no subdivision belongs to two (vanilla's
/// `mp_darkvalley`, `mp_firestation` and `zaton`).
///
/// Each model comes back as an object of its subdivisions in its own space, flagged `eoMultipleUsage`, dressed as its
/// first placement lies on the collision form; each placement as a scene object of its matrix, in its sector. Where
/// that placement's `MT_LOD` names the model's strip in the level's impostor atlas, the strips are cut back out as the
/// textures the Build reads (`lod\lod_<model>`) and the model is flagged `eoUsingLOD`.
pub(crate) struct LevelTreeConversion;

/// What a level holds of its trees' impostors: the atlas, and each `MT_LOD`'s definition by its visual's index.
pub(crate) struct LevelTreeImpostors<'a> {
  pub(crate) atlas: &'a LevelTreeImpostorAtlas,
  pub(crate) lods: HashMap<u32, &'a OgfLodDefinitionChunk>,
}

/// One tree visual of a sector, its geometry in its model's own space.
pub(crate) struct LevelTreeVisual {
  /// The visual's `MT_LOD` parent, which names its placement, where it has one.
  pub(crate) parent: Option<u32>,
  pub(crate) sector: u16,
  /// Its vertex buffer, first vertex, index buffer and first index: which subdivision of which model it draws.
  pub(crate) subdivision: [u32; 4],
  pub(crate) placement: Matrix4x4,
  pub(crate) geometry: LevelVisualGeometry,
}

/// A level's trees decompiled: an object per model, and each placement of one.
pub(crate) struct LevelTreeDecompile {
  /// The models, each at the origin of its own space.
  pub(crate) models: Vec<LevelDecompiledObject>,
  pub(crate) placements: Vec<LevelTreePlacement>,
  /// Each model's impostor strips, by its reference, where it has one.
  pub(crate) impostors: Vec<(String, LevelTreeImpostor)>,
  /// The impostors that did not cut, the model coming back without one.
  pub(crate) refused: Vec<LevelSceneRefusal>,
}

/// A placed tree: the model it places by reference, the mesh its sector names, its transform and sector.
pub(crate) struct LevelTreePlacement {
  pub(crate) reference: String,
  pub(crate) mesh: String,
  pub(crate) transform: EditableObjectTransform,
  pub(crate) sector: u16,
}

impl LevelTreeConversion {
  /// The tree visual `visual` is, in sector `sector` under the `MT_LOD` `parent` where one parents it, with `geometry`
  /// read from the level's buffers; `None` for a visual that is no tree.
  pub(crate) fn read(
    visual: &LevelVisual,
    sector: u16,
    parent: Option<u32>,
    geometry: LevelVisualGeometry,
  ) -> Option<LevelTreeVisual> {
    let (tree, container) = (visual.tree.as_ref()?, visual.geometry.as_ref()?);

    Some(LevelTreeVisual {
      parent,
      sector,
      subdivision: [
        container.vertex_buffer_id,
        container.vertex_base,
        container.index_buffer_id,
        container.index_base,
      ],
      placement: tree.transform,
      geometry,
    })
  }

  /// The models and placements of `visuals`, each model named `levels\<name>\<name>_tree_<N>` in the order its first
  /// placement comes; each model dressed by `surfaces` from where its first placement lies on `collision`, and given
  /// its impostor from `impostors` where that placement's `MT_LOD` names one.
  ///
  /// # Errors
  ///
  /// Returns an error when a model's geometry does not decompile or a placement's matrix is not a transform.
  pub(crate) fn decompile(
    name: &str,
    visuals: Vec<LevelTreeVisual>,
    mut surfaces: Option<&mut LevelSurfaceInference>,
    mut collision: Option<&mut LevelCollisionIndex>,
    impostors: Option<&LevelTreeImpostors<'_>>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<LevelTreeDecompile> {
    // Each placement in order of its first visual, with its visuals' subdivisions sorted.
    let mut placements: Vec<(u16, Matrix4x4, Vec<LevelTreeVisual>)> = Vec::new();
    let mut by_key: BTreeMap<PlacementKey, usize> = BTreeMap::new();

    for visual in visuals {
      let key: PlacementKey = match visual.parent {
        Some(parent) => PlacementKey::Parent(parent),
        None => PlacementKey::Matrix(visual.sector, Self::get_matrix_bits(&visual.placement)),
      };
      let index: usize = *by_key.entry(key).or_insert_with(|| {
        placements.push((visual.sector, visual.placement, Vec::new()));
        placements.len() - 1
      });

      placements[index].2.push(visual);
    }

    let mut decompile: LevelTreeDecompile = LevelTreeDecompile {
      models: Vec::new(),
      placements: Vec::with_capacity(placements.len()),
      impostors: Vec::new(),
      refused: Vec::new(),
    };
    // Each model's reference and mesh, by its subdivisions.
    let mut models: BTreeMap<Vec<[u32; 4]>, (String, String)> = BTreeMap::new();

    for (sector, placement, mut visuals) in placements {
      visuals.sort_by_key(|visual| visual.subdivision);

      let subdivisions: Vec<[u32; 4]> = visuals.iter().map(|visual| visual.subdivision).collect();
      let (reference, mesh) = match models.get(&subdivisions) {
        Some(model) => model.clone(),
        None => {
          let reference: String = format!("levels\\{name}\\{name}_tree_{}", models.len());
          let parent: Option<u32> = visuals.first().and_then(|visual| visual.parent);
          let geometry: Vec<LevelVisualGeometry> = visuals
            .into_iter()
            .flat_map(|visual| {
              Self::dress(
                visual.geometry,
                &placement,
                surfaces.as_deref_mut(),
                collision.as_deref_mut(),
              )
            })
            .collect();
          let mut decompiled = ObjectDecompiler::decompile_level_geometry(&geometry)?;

          decompiled.object.flags |= EditableObjectFile::MULTIPLE_USAGE_FLAG;

          if let Some((impostors, lod)) =
            impostors.and_then(|impostors| Some((impostors, *impostors.lods.get(&parent?)?)))
          {
            match impostors.atlas.cut(lod) {
              Ok(impostor) => {
                decompiled.object.flags |= EditableObjectFile::USING_LOD_FLAG;
                decompile.impostors.push((reference.clone(), impostor));

                if !Self::fit_impostor_box(&mut decompiled.object, lod, &placement) {
                  decompile.refused.push(LevelSceneRefusal {
                    file: String::from(LevelFile::FILE_NAME),
                    object: reference.clone(),
                    reason: String::from(
                      "The impostor's quads stand in no box around the model, so the Build stands them around the \
                       model's own",
                    ),
                  });
                }
              }
              Err(error) => decompile.refused.push(LevelSceneRefusal {
                file: String::from(LevelTreeImpostorAtlas::COLOR_FILE_NAME),
                object: reference.clone(),
                reason: format!("The impostor did not cut, so the model comes back without one: {error}"),
              }),
            }
          }

          let mesh: String = decompiled
            .object
            .meshes
            .iter()
            .flatten()
            .next()
            .map(|mesh| mesh.name.clone())
            .unwrap_or_default();

          decompile.models.push(LevelDecompiledObject {
            reference: reference.clone(),
            transform: EditableObjectTransform::default(),
            source: String::from(LevelFile::FILE_NAME),
            decompile: decompiled,
          });
          models.insert(subdivisions, (reference.clone(), mesh.clone()));
          (reference, mesh)
        }
      };

      context.names.reserve(&reference);
      decompile.placements.push(LevelTreePlacement {
        reference,
        mesh,
        transform: EditableObjectTransform::from_matrix_as_saved(&placement)?,
        sector,
      });
    }

    Ok(decompile)
  }

  /// Widens `object`'s first mesh's stored box to the box its impostor `lod` was stood in by `placement`, as the Build
  /// stands it in the union of the meshes' boxes (`CEditableObject::GetLODFrame`), which the editor reads as saved
  /// (`CEditableMesh::LoadMesh`). The source object's box held geometry the level does not draw, such as a collision
  /// trunk under the ground; so the rebuilt quads stand where the shipped ones did. The box across is known only as
  /// its widest half side, which goes on the model's own widest axis.
  ///
  /// Returns `false`, leaving the box as it was, where `placement` does not invert or the recovered box does not hold
  /// the model's.
  pub(crate) fn fit_impostor_box(
    object: &mut EditableObjectFile,
    lod: &OgfLodDefinitionChunk,
    placement: &Matrix4x4,
  ) -> bool {
    /// How far the model's box may stand past the recovered one, past rounding.
    const TOLERANCE: f32 = 1e-3;

    let Some(inverse) = placement.invert_affine() else {
      return false;
    };
    let Some(held) = object
      .meshes
      .iter()
      .flatten()
      .map(|mesh| mesh.bounds)
      .reduce(|mut held, bounds| {
        held.include_box(&bounds);
        held
      })
    else {
      return false;
    };
    let corners: [Vector3d; 4] = lod.facets[0]
      .vertices
      .each_ref()
      .map(|vertex| inverse.transform_point(&vertex.position));
    let (center, across, half_height) = LevelLodStrip::get_first_frame_extent(&corners);
    let reach = |min: f32, max: f32, middle: f32| (middle - min).max(max - middle);
    let (half_x, half_z) = (
      reach(held.min.x, held.max.x, center.x),
      reach(held.min.z, held.max.z, center.z),
    );
    let (half_x, half_z) = if held.max.x - held.min.x >= held.max.z - held.min.z {
      (across, half_z)
    } else {
      (half_x, across)
    };
    let fitted: Aabb = Aabb {
      min: Vector3d::new(center.x - half_x, center.y - half_height, center.z - half_z),
      max: Vector3d::new(center.x + half_x, center.y + half_height, center.z + half_z),
    };
    let holds: bool = [
      (fitted.min.x, held.min.x),
      (fitted.min.y, held.min.y),
      (fitted.min.z, held.min.z),
    ]
    .iter()
    .all(|(fitted, held)| *fitted <= held + TOLERANCE)
      && [
        (fitted.max.x, held.max.x),
        (fitted.max.y, held.max.y),
        (fitted.max.z, held.max.z),
      ]
      .iter()
      .all(|(fitted, held)| *fitted >= held - TOLERANCE);

    if !holds {
      return false;
    }

    if let Some(mesh) = object.meshes.iter_mut().flatten().next() {
      mesh.bounds.include_box(&fitted);
    }

    true
  }

  /// `geometry` in its model's space dressed as it lies placed by `placement`: split by the game material its placed
  /// triangles lie on, each part keeping the model's own vertices; as read where there is no inference.
  fn dress(
    geometry: LevelVisualGeometry,
    placement: &Matrix4x4,
    surfaces: Option<&mut LevelSurfaceInference>,
    collision: Option<&mut LevelCollisionIndex>,
  ) -> Vec<LevelVisualGeometry> {
    let (Some(surfaces), Some(collision)) = (surfaces, collision) else {
      return vec![geometry];
    };
    let mut placed: LevelVisualGeometry = geometry.clone();

    for vertex in &mut placed.vertices {
      vertex.position = placement.transform_point(&vertex.position);
    }

    surfaces
      .dress(collision, placed)
      .into_iter()
      .map(|part| LevelVisualGeometry {
        vertices: geometry.vertices.clone(),
        ..part
      })
      .collect()
  }

  fn get_matrix_bits(matrix: &Matrix4x4) -> [u32; Matrix4x4::ELEMENTS] {
    matrix.values.map(f32::to_bits)
  }
}

/// What groups a placement's visuals: its `MT_LOD` parent, or its sector and matrix where it has none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PlacementKey {
  Parent(u32),
  Matrix(u16, [u32; Matrix4x4::ELEMENTS]),
}
