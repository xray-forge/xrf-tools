//! Pins what an SDK object is described as beyond the model compiled from it.
//!
//! The compile itself is `xrf-rawdata`'s to pin.

use xrf_app_core::assets::AssetSource;
use xrf_chunk::ChunkLayout;
use xrf_editable_object::{EditableFace, EditableMesh, EditableObjectFile, EditableSurface, EditableSurfaceFaces};
use xrf_math::{Aabb, Vector3d};
use xrf_rawdata::{ObjectInference, ObjectProvenance};

use crate::{SelectedObject, SelectedObjectDescription, SelectedObjectSurface};

fn surface(name: &str, flags: u32) -> EditableSurface {
  EditableSurface {
    name: String::from(name),
    shader: String::from("models\\model"),
    compiler_shader: String::from("default"),
    game_material: String::from("objects\\wood"),
    texture: format!("wood\\{name}"),
    vmap: String::from("Texture"),
    flags,
    fvf: EditableSurface::POSITION_NORMAL_UV_FVF,
    texture_coordinate_count: 1,
  }
}

/// A mesh of four points and three faces, two on `planks` and one on `nails`, in two smoothing groups.
fn mesh() -> EditableMesh {
  EditableMesh {
    name: String::from("skin"),
    bounds: Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::new(1.0, 1.0, 0.0),
    },
    flags: EditableMesh::VISIBLE_FLAG,
    options: None,
    points: vec![Vector3d::ZERO; 4],
    has_point_faces: false,
    faces: vec![EditableFace::default(); 3],
    smoothing_groups: Some(vec![1, 1, 2]),
    corner_normals: None,
    vmap_references: Vec::new(),
    surface_faces: vec![
      EditableSurfaceFaces {
        surface: String::from("planks"),
        faces: vec![0, 1],
      },
      EditableSurfaceFaces {
        surface: String::from("nails"),
        faces: vec![2],
      },
    ],
    vmaps: None,
    layout: ChunkLayout::default(),
  }
}

fn object() -> EditableObjectFile {
  EditableObjectFile {
    flags: EditableObjectFile::DYNAMIC_FLAG,
    flags_size: EditableObjectFile::FLAGS_SIZE,
    class_script: Some(String::new()),
    lods: Some(String::from("dynamics\\crate_lod")),
    meshes: Some(vec![mesh()]),
    surfaces: Some(vec![
      surface("planks", 0),
      surface("nails", EditableSurface::TWO_SIDED_FLAG),
    ]),
    bones: None,
    embedded_motions: None,
    motion_references: Some(vec![String::from("crate_motions")]),
    bone_parts: None,
    actor_transform: None,
    description: None,
    layout: ChunkLayout::default(),
  }
}

#[test]
fn describes_what_only_the_object_holds() {
  let provenance: ObjectProvenance = ObjectProvenance::new(
    "meshes/crate.ogf",
    b"ogf",
    "xrf-cli object decompile",
    "1.0.0",
    vec![ObjectInference::SurfaceCompilerShaders],
  );
  let description: SelectedObjectDescription = SelectedObjectDescription::of(&object(), Some(&provenance));

  assert_eq!(
    description
      .surfaces
      .iter()
      .map(|it| (it.name.as_str(), it.face_count, it.is_two_sided))
      .collect::<Vec<_>>(),
    [("planks", 2, false), ("nails", 1, true)]
  );
  assert_eq!(
    description.surfaces[0],
    SelectedObjectSurface {
      name: String::from("planks"),
      texture: String::from("wood\\planks"),
      shader: String::from("models\\model"),
      compiler_shader: String::from("default"),
      game_material: String::from("objects\\wood"),
      is_two_sided: false,
      face_count: 2,
    }
  );
  assert_eq!(
    (
      description.mesh_count,
      description.point_count,
      description.face_count,
      description.smoothing_group_count
    ),
    (1, 4, 3, 2)
  );
  assert_eq!(description.motion_references, ["crate_motions"]);
  assert!(!description.has_embedded_motions);
  // An empty class script is the editor's way of storing none.
  assert_eq!(description.class_script, None);
  assert_eq!(description.lods.as_deref(), Some("dynamics\\crate_lod"));
  assert_eq!(
    description.provenance.map(|it| (it.source, it.inferred)),
    Some((
      String::from("meshes/crate.ogf"),
      vec![ObjectInference::SurfaceCompilerShaders]
    ))
  );
}

#[test]
fn tells_an_object_from_a_compiled_model_by_its_extension() {
  let file = |path: &str| AssetSource::File {
    path: String::from(path),
  };

  assert!(SelectedObject::is_object(&file("rawdata\\crate.object")));
  assert!(SelectedObject::is_object(&file("rawdata\\CRATE.OBJECT")));
  assert!(!SelectedObject::is_object(&file("meshes\\crate.ogf")));
  assert!(!SelectedObject::is_object(&file("object")));
}
