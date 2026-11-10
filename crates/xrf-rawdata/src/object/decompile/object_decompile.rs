use std::collections::HashMap;

use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface};

use crate::object::compile::object_compile_options::TangentMethod;
use crate::object::decompile::object_inference::ObjectInference;
use crate::object::detail_parameters::DetailParameters;

/// An OGF decompiled into an editable object, and what in it was inferred rather than read.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectDecompile {
  pub object: EditableObjectFile,
  /// Each kind of inference the object carries, once, in a fixed order.
  pub inferred: Vec<ObjectInference>,
  /// The tangent method the OGF's basis was made with, by the exporter its description names, which a compile follows
  /// to give the basis back.
  pub tangents: TangentMethod,
  /// What a detail model decompiled carried beside its geometry, which a compile back into a `.dm` writes again; `None`
  /// for an OGF.
  pub detail: Option<DetailParameters>,
}

impl ObjectDecompile {
  /// One object holding each part's meshes under the name paired with it, the first part's object the rest join: a
  /// surface the same as one already held but for its name takes that one's, another whose name is taken is numbered
  /// `_1` upwards, and each mesh's faces follow their surface's name. Inferences are joined. `None` for no part.
  pub(crate) fn merge_meshes(parts: Vec<(String, Self)>) -> Option<Self> {
    let mut parts = parts.into_iter();
    let (name, mut merged) = parts.next()?;
    let mut surfaces: Vec<EditableSurface> = merged.object.surfaces.take().unwrap_or_default();
    let mut meshes: Vec<EditableMesh> = merged.object.meshes.take().unwrap_or_default();

    for mesh in &mut meshes {
      mesh.name.clone_from(&name);
    }

    for (name, part) in parts {
      let mut renamed: HashMap<String, String> = HashMap::new();

      for surface in part.object.surfaces.unwrap_or_default() {
        let held: Option<&EditableSurface> = surfaces.iter().find(|it| {
          EditableSurface {
            name: surface.name.clone(),
            ..(*it).clone()
          } == surface
        });
        let target: String = match held {
          Some(held) => held.name.clone(),
          None => {
            let target: String = Self::get_free_name(&surfaces, &surface.name);

            surfaces.push(EditableSurface {
              name: target.clone(),
              ..surface.clone()
            });
            target
          }
        };

        renamed.insert(surface.name, target);
      }

      for mut mesh in part.object.meshes.unwrap_or_default() {
        mesh.name.clone_from(&name);

        for faces in &mut mesh.surface_faces {
          if let Some(target) = renamed.get(&faces.surface) {
            faces.surface.clone_from(target);
          }
        }

        meshes.push(mesh);
      }

      merged.inferred.extend(part.inferred);
    }

    merged.inferred.sort_unstable();
    merged.inferred.dedup();
    merged.object.surfaces = Some(surfaces);
    merged.object.meshes = Some(meshes);

    Some(merged)
  }

  /// `name`, or it with the first free `_n` where a surface already has it.
  fn get_free_name(surfaces: &[EditableSurface], name: &str) -> String {
    let is_taken = |candidate: &str| surfaces.iter().any(|surface| surface.name == candidate);

    if !is_taken(name) {
      return String::from(name);
    }

    (1..)
      .map(|number| format!("{name}_{number}"))
      .find(|candidate| !is_taken(candidate))
      .expect("some numbered name is free")
  }
}
