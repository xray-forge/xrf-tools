use xrf_editable_level::EditableObjectTransform;
use xrf_error::XrfResult;

use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::instance::level_instance_matcher::LevelInstanceMatcher;
use crate::level::instance::level_instance_triangles::LevelInstanceTriangles;
use crate::level::instance::level_library_object::LevelLibraryTriangle;
use crate::level::instance::level_tree_match::LevelTreeMatch;
use crate::level::level_geometry_conversion::LevelDecompiledObject;
use crate::level::level_tree_conversion::LevelTreeDecompile;

/// The tree models of a level that a library's trees draw, found as a static object is found ([`LevelInstanceMatcher`],
/// the model standing for the level).
pub(crate) struct LevelTreeNaming;

impl LevelTreeNaming {
  /// Each model of `trees` one of `library`'s trees stands for whole: the model left out, its impostor strips named for
  /// the library tree, and each placement of it placing the library tree instead, composed with the fit.
  ///
  /// # Errors
  ///
  /// Returns an error when a placement composed with a fit is no transform the editor saves.
  pub(crate) fn name(library: &LevelInstanceLibrary, trees: &mut LevelTreeDecompile) -> XrfResult<Vec<LevelTreeMatch>> {
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

      for placement in trees.placements.iter_mut().filter(|it| it.reference == model.reference) {
        placement.transform =
          EditableObjectTransform::from_matrix_as_saved(&placement.transform.to_matrix().multiply(&fit))?;
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

    Ok(matches)
  }

  /// Gives the strips cut for `model` to the library tree of `reference` that stands for it. The Build loads a
  /// placement's strips by its object's library name (`CEditableObject::GetLODTextureName`,
  /// `ECore/Editor/EditObjectEditor.cpp`; `SceneBuilder::BuildObjectLOD`, `LevelEditor/Edit/BuilderLODs.cpp`), and the
  /// level's atlas holds the very strips it loaded for that tree; a library's objects need not ship them, as the game's
  /// textures do not. A tree standing for two models keeps the first one's.
  fn rename_impostor(trees: &mut LevelTreeDecompile, model: &str, reference: &str) {
    let is_held: bool = trees.impostors.iter().any(|(held, _)| held == reference);

    if is_held {
      trees.impostors.retain(|(held, _)| held != model);
    } else if let Some((held, _)) = trees.impostors.iter_mut().find(|(held, _)| held == model) {
      reference.clone_into(held);
    }
  }
}
