use xrf_app_core::TauriResult;
use xrf_editable_object::{EditableObjectFile, EditableOccluder};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_object_visual::{ObjectVisual, VisualSurfaces};
use xrf_rawdata::{ObjectCompileOptions, ObjectCompileSource};
use xrf_renderer::RenderSpawnModel;
use xrf_visual::VisualPoser;

use crate::scene::{
  SceneDrawnObject, SceneObjectModels, SceneObjectParts, SceneOccluderLook, SceneSurfaceKind, SceneSurfaceKinds,
};

/// How a scene compiles its library objects to models: each surface dressed as the scene's game data answers it, each
/// kind of surface ([`SceneSurfaceKind`]) parted into a model of its own, and an occluder drawn whole as its fill, as
/// the editor draws it whatever its surfaces.
pub(crate) struct SceneObjectCompiler<'a> {
  /// How the scene's game data dresses a surface.
  pub surfaces: &'a XraySurfaceResolver,
  /// Which kind each surface is.
  pub kinds: &'a SceneSurfaceKinds,
}

impl SceneObjectCompiler<'_> {
  /// The models of `kinds` that `file`, the library object `object` places, compiles to; those of other kinds are not
  /// compiled, so a batch asking for some of an object's models compiles no more than it asks.
  ///
  /// # Errors
  ///
  /// Returns why a part of the library object does not compile.
  pub(crate) fn compile(
    &self,
    object: &SceneDrawnObject,
    file: EditableObjectFile,
    kinds: &[SceneSurfaceKind],
  ) -> TauriResult<SceneObjectModels> {
    let occluder: Option<EditableOccluder> = file.get_occluder();
    let mut parts: SceneObjectParts = match occluder {
      Some(_) => SceneObjectParts::whole(SceneSurfaceKind::Drawn, file),
      None => self.kinds.split(file, kinds),
    };
    let part: ScenePartCompile<'_> = ScenePartCompile {
      object,
      surfaces: self.surfaces,
      occluder,
    };
    let mut models: SceneObjectModels = SceneObjectModels::default();

    for kind in kinds {
      if let Some(model) = part.compile(kind.format_model_name(&object.reference), parts.take(*kind))? {
        models.insert(*kind, model);
      }
    }

    Ok(models)
  }
}

/// What every part of one scene object's library object compiles with: the object, how its surfaces are dressed, and
/// the occluder whose fill dresses them instead.
struct ScenePartCompile<'a> {
  object: &'a SceneDrawnObject,
  surfaces: &'a XraySurfaceResolver,
  occluder: Option<EditableOccluder>,
}

impl ScenePartCompile<'_> {
  /// The model `part`, a part of the library object, compiles to, posed as stored and named `name`, each of its
  /// submeshes dressed as the surfaces answer it, or as the occluder's fill; none for no part.
  ///
  /// # Errors
  ///
  /// Returns why the part does not compile.
  fn compile(&self, name: String, part: Option<EditableObjectFile>) -> TauriResult<Option<RenderSpawnModel>> {
    let Some(part) = part else {
      return Ok(None);
    };
    let Self {
      object,
      surfaces,
      occluder,
    } = self;
    // A scene's object compiles with the CLI's default tangent method, which only a skeleton's compile reads.
    let mut visual: ObjectVisual = ObjectVisual::compile(
      &part,
      &ObjectCompileSource {
        source_file: object.logical_path.clone(),
        convertor: String::from("xrf-app levels"),
        built_at: 0,
      },
      &ObjectCompileOptions::default(),
    )
    .map_err(|error| format!("Library object '{}' does not compile: {error}", object.logical_path))?;

    // Let go before posing copies the package, which keeps a large shell's peak to the model and its copy.
    drop(part);

    let surfaces: Vec<XraySurfaceDescriptor> = match occluder {
      Some(occluder) => SceneOccluderLook::dress(*occluder, &mut visual.package.description),
      None => VisualSurfaces::describe(surfaces, &visual.package.description),
    };

    Ok(Some(RenderSpawnModel {
      name,
      package: VisualPoser::pose(&visual.package, None),
      surfaces,
      skeleton: None,
    }))
  }
}
