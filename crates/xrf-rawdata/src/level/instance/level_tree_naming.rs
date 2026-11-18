use xrf_editable_level::EditableObjectTransform;
use xrf_error::XrfResult;

use crate::level::geometry::level_decompiled_object::LevelDecompiledObject;
use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::instance::level_instance_matcher::LevelInstanceMatcher;
use crate::level::instance::level_instance_triangles::LevelInstanceTriangles;
use crate::level::instance::level_library_triangle::LevelLibraryTriangle;
use crate::level::instance::level_tree_match::LevelTreeMatch;
use crate::level::trees::level_tree_decompile::LevelTreeDecompile;

/// Finds the level's tree models that library trees draw, searching each model as [`LevelInstanceMatcher`] searches a
/// level.
pub(crate) struct LevelTreeNaming;

impl LevelTreeNaming {
  /// Replaces each model of `trees` that a library tree stands for whole: its placements place the library tree
  /// instead, and its impostor strips are named for it. A model any of whose placements composes into no transform the
  /// editor saves stays as decompiled.
  pub(crate) fn name(library: &LevelInstanceLibrary, trees: &mut LevelTreeDecompile) -> Vec<LevelTreeMatch> {
    let mut matches: Vec<LevelTreeMatch> = Vec::new();
    let mut kept: Vec<LevelDecompiledObject> = Vec::with_capacity(trees.models.len());

    for model in std::mem::take(&mut trees.models) {
      let found = library
        .get_object(model.reference.clone(), &model.decompile.object)
        .and_then(|object| {
          let mut index: LevelInstanceTriangles = LevelInstanceTriangles::of_object(&object);
          let found = LevelInstanceMatcher::find(&library.trees, &mut index, LevelInstanceMatcher::COVERAGE);
          let claimed: f64 = index
            .triangles
            .iter()
            .filter(|triangle| triangle.is_claimed)
            .map(|triangle| LevelLibraryTriangle::get_area(&triangle.points))
            .sum();

          // The library tree must stand for the whole model, not a branch of it.
          (claimed >= LevelInstanceMatcher::COVERAGE * object.area)
            .then(|| found.into_iter().next())
            .flatten()
        });
      let Some(found) = found else {
        kept.push(model);
        continue;
      };
      let tree = &library.trees[found.object];
      let fit = found.fit.to_matrix();
      // An uneven scale and the library tree's turn compose into a shear no scene object holds; the model then stays.
      let Ok(placed) = trees
        .placements
        .iter()
        .enumerate()
        .filter(|(_, placement)| placement.reference == model.reference)
        .map(|(index, placement)| {
          EditableObjectTransform::from_matrix_as_saved(&placement.transform.to_matrix().multiply(&fit))
            .map(|transform| (index, transform))
        })
        .collect::<XrfResult<Vec<(usize, EditableObjectTransform)>>>()
      else {
        kept.push(model);
        continue;
      };

      for (index, transform) in placed {
        let placement = &mut trees.placements[index];

        placement.transform = transform;
        placement.reference.clone_from(&tree.reference);

        if let Some(mesh) = tree.meshes.first() {
          placement.mesh.clone_from(mesh);
        }
      }

      Self::rename_impostor(trees, &model.reference, &tree.reference);
      matches.push(LevelTreeMatch {
        model: model.reference,
        reference: tree.reference.clone(),
        path: tree.path.clone(),
        coverage: found.coverage,
        error: found.error,
      });
    }

    trees.models = kept;

    matches
  }

  /// Gives the strips cut for `model` to the library tree `reference` standing for it; a tree standing for two models
  /// keeps the first one's.
  ///
  /// The Build loads strips by the library name (`CEditableObject::GetLODTextureName`), and libraries need not ship
  /// them, so the strips the atlas holds for that tree are the ones it needs.
  fn rename_impostor(trees: &mut LevelTreeDecompile, model: &str, reference: &str) {
    let is_held: bool = trees.impostors.iter().any(|(held, _)| held == reference);

    if is_held {
      trees.impostors.retain(|(held, _)| held != model);
    } else if let Some((held, _)) = trees.impostors.iter_mut().find(|(held, _)| held == model) {
      reference.clone_into(held);
    }
  }
}
