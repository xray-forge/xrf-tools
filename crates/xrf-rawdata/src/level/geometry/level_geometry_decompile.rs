use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditablePortalObject, EditableSceneObject, EditableSectorObject,
};

use crate::level::geometry::level_decompiled_object::LevelDecompiledObject;
use crate::level::geometry::level_geometry_census::LevelGeometryCensus;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;
use crate::level::instance::level_tree_match::LevelTreeMatch;
use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// A level's geometry decompiled: the objects to write, and the parts placing them and culling them.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelGeometryDecompile {
  pub objects: Vec<LevelDecompiledObject>,
  pub scene_objects: EditableObjectPart<EditableSceneObject>,
  /// Empty where a sector draws no static geometry, with the portals: see
  /// [`LevelGeometryConversion::decompile`](crate::parts::LevelGeometryConversion::decompile).
  pub sectors: EditableObjectPart<EditableSectorObject>,
  pub portals: EditableObjectPart<EditablePortalObject>,
  pub census: LevelGeometryCensus,
  /// The library objects placed by reference, which the scene needs beside it.
  pub instances: Vec<LevelInstancePlacement>,
  /// The library trees found standing for tree models, which the scene places by reference instead.
  pub trees: Vec<LevelTreeMatch>,
  /// The tree models' and library trees' impostor strips, which the Build loads for them.
  pub textures: Vec<LevelSceneTextureFile>,
  /// Each sector left out, and each impostor that did not cut, and why.
  pub refused: Vec<LevelSceneRefusal>,
  /// The names of the spawn list's packets the objects stand for, which the Build writes again from them, so no spawn
  /// point should.
  pub folded_entities: Vec<String>,
}

impl LevelGeometryDecompile {
  /// Adds `object` with a scene object placing it at its transform under its reference, which `context` keeps from
  /// every name generated after.
  pub(super) fn place(&mut self, object: LevelDecompiledObject, context: &mut LevelDecompileContext) {
    context.names.reserve(&object.reference);
    self.scene_objects.objects.push(EditableSceneObject {
      object: EditableCustomObject::new(object.reference.clone(), object.transform),
      reference: object.reference.clone(),
      flags: 0,
    });
    self.objects.push(object);
  }
}
