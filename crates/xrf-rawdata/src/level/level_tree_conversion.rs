use std::collections::BTreeMap;

use xrf_editable_level::EditableObjectTransform;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelVisual};
use xrf_math::Matrix4x4;

use crate::level::level_collision_index::LevelCollisionIndex;
use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_geometry_conversion::LevelDecompiledObject;
use crate::level::level_surface_inference::LevelSurfaceInference;
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A level's trees folded back into the multi-use objects the Build placed them from (`SceneBuilder::BuildMUObject`).
/// xrLC drew each placement as one tree visual (`MT_TREE_ST`, `MT_TREE_PM`) per subdivision of its model, in the model's
/// own space with the placement's matrix (`OGF_TREEDEF2`), under one `MT_LOD` where the model had an impostor
/// (`xrMU_Model_export_OGF.cpp`). So a placement is the visuals one `MT_LOD` parents, or that share a matrix where none
/// does, and a model is the subdivisions every placement of it names: no subdivision belongs to two (vanilla's
/// `mp_darkvalley`, `mp_firestation` and `zaton`).
///
/// Each model comes back as an object of its subdivisions in its own space, flagged `eoMultipleUsage`, dressed as its
/// first placement lies on the collision form; each placement as a scene object of its matrix, in its sector. The
/// impostor is not kept, since the editor's Build needs the model's own LOD textures for one.
pub(crate) struct LevelTreeConversion;

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
  /// placement comes; each model dressed by `surfaces` from where its first placement lies on `collision`.
  ///
  /// # Errors
  ///
  /// Returns an error when a model's geometry does not decompile or a placement's matrix is not a transform.
  pub(crate) fn decompile(
    name: &str,
    visuals: Vec<LevelTreeVisual>,
    mut surfaces: Option<&mut LevelSurfaceInference>,
    mut collision: Option<&mut LevelCollisionIndex>,
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
