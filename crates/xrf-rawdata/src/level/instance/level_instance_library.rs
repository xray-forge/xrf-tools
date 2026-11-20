use std::collections::HashMap;
use std::path::PathBuf;

use glam::{DVec2, DVec3};
use xrf_editable_object::{EditableFaceCorner, EditableMesh, EditableObjectFile, EditableSurface, EditableVMapKind};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::level::instance::level_library_object::LevelLibraryObject;
use crate::level::instance::level_library_triangle::LevelLibraryTriangle;

/// The library objects a decompile looks for in a level: each added object the Build merges into the geometry with a
/// surface that renders, multi-use ones kept apart as trees, and the textures they draw, each numbered once.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelInstanceLibrary {
  /// The objects the Build merges into the static geometry, and the multi-use ones it builds once and references; each
  /// list the largest first.
  pub objects: Vec<LevelLibraryObject>,
  pub trees: Vec<LevelLibraryObject>,
  /// Textures by their lowercased name.
  textures: HashMap<String, u32>,
}

impl LevelInstanceLibrary {
  /// Object flags that keep an object out of the level's geometry.
  const UNBUILT_FLAGS: u32 =
    EditableObjectFile::DYNAMIC_FLAG | EditableObjectFile::HOM_FLAG | EditableObjectFile::SOUND_OCCLUDER_FLAG;

  /// Orders the objects and trees as they are tried, largest first, then by reference; call once all are added.
  pub fn sort(&mut self) {
    for objects in [&mut self.objects, &mut self.trees] {
      objects.sort_by(|left, right| {
        right
          .area
          .total_cmp(&left.area)
          .then_with(|| left.reference.cmp(&right.reference))
      });
    }
  }

  /// The number `texture` takes in the table, any case; none where no library object draws it.
  pub fn get_texture(&self, texture: &str) -> Option<u32> {
    self.textures.get(&texture.to_lowercase()).copied()
  }

  /// Adds `object` where the Build builds it and it renders: to the trees where it is multi-use, else to the objects.
  /// A surface whose compiler shader `compiler_shaders` lacks, or with none given, is taken to draw and collide.
  pub fn add(
    &mut self,
    reference: String,
    path: PathBuf,
    object: &EditableObjectFile,
    compiler_shaders: Option<&ShaderCompilerFile>,
  ) {
    if object.flags & Self::UNBUILT_FLAGS != 0 {
      return;
    }

    let Some(read) = Self::read_object(reference, path, object, compiler_shaders, &mut |texture| {
      Some(self.intern(texture))
    }) else {
      return;
    };

    if object.flags & EditableObjectFile::MULTIPLE_USAGE_FLAG != 0 {
      self.trees.push(read);
    } else {
      self.objects.push(read);
    }
  }

  /// A decompiled model as the library would hold it, to look for library objects in; surfaces of textures no library
  /// object draws are left out.
  pub(crate) fn get_object(&self, reference: String, object: &EditableObjectFile) -> Option<LevelLibraryObject> {
    Self::read_object(reference, PathBuf::new(), object, None, &mut |texture| {
      self.get_texture(texture)
    })
  }

  /// `object`'s rendered triangles, each surface's texture numbered by `number` (left out where it gives none), and
  /// its collision-only ones; none where nothing renders.
  fn read_object(
    reference: String,
    path: PathBuf,
    object: &EditableObjectFile,
    compiler_shaders: Option<&ShaderCompilerFile>,
    number: &mut dyn FnMut(&str) -> Option<u32>,
  ) -> Option<LevelLibraryObject> {
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();
    let meshes: &[EditableMesh] = object.meshes.as_deref().unwrap_or_default();
    let mut triangles: Vec<LevelLibraryTriangle> = Vec::new();
    let mut collision: Vec<[DVec3; 3]> = Vec::new();
    let flags = |name: &str| -> u32 {
      compiler_shaders.and_then(|shaders| shaders.find_shader(name)).map_or(
        ShaderCompilerShader::COLLISION | ShaderCompilerShader::RENDERING,
        |shader| shader.flags,
      )
    };

    for (index, mesh) in meshes.iter().enumerate() {
      for faces in &mesh.surface_faces {
        let Some(surface) = surfaces.iter().find(|surface| surface.name == faces.surface) else {
          continue;
        };
        let surface_flags: u32 = flags(&surface.compiler_shader);
        let corners = faces
          .faces
          .iter()
          .filter_map(|face| mesh.faces.get(*face as usize))
          .filter_map(|face| Some((Self::get_points(mesh, face.corners.map(|corner| corner.point))?, face)));

        if surface_flags & ShaderCompilerShader::RENDERING == 0 {
          if surface_flags & ShaderCompilerShader::COLLISION != 0 {
            collision.extend(corners.map(|(points, _)| points));
          }

          continue;
        }

        let Some(texture) = Some(&surface.texture)
          .filter(|texture| !texture.is_empty())
          .and_then(|texture| number(texture))
        else {
          continue;
        };

        for (points, face) in corners {
          let area: f64 = LevelLibraryTriangle::get_area(&points);

          if area > 0.0 {
            triangles.push(LevelLibraryTriangle {
              mesh: index,
              texture,
              points,
              uvs: Self::get_uvs(mesh, &face.corners),
              is_two_sided: surface.is_two_sided(),
              area,
            });
          }
        }
      }
    }

    if triangles.is_empty() {
      return None;
    }

    triangles.sort_by(|left, right| right.area.total_cmp(&left.area));

    Some(LevelLibraryObject {
      reference,
      path,
      meshes: meshes.iter().map(|mesh| mesh.name.clone()).collect(),
      area: triangles.iter().map(|triangle| triangle.area).sum(),
      triangles,
      collision,
    })
  }

  /// A face's corners in the mesh's space, none where a corner names a point the mesh does not hold.
  fn get_points(mesh: &EditableMesh, corners: [u32; 3]) -> Option<[DVec3; 3]> {
    let point = |index: u32| -> Option<DVec3> {
      let point = mesh.points.get(index as usize)?;

      Some(DVec3::new(f64::from(point.x), f64::from(point.y), f64::from(point.z)))
    };

    Some([point(corners[0])?, point(corners[1])?, point(corners[2])?])
  }

  /// A face's corners' texture coordinates, each from the first map of two coordinates its vertex map reference names;
  /// none where a corner has none.
  fn get_uvs(mesh: &EditableMesh, corners: &[EditableFaceCorner; 3]) -> Option<[DVec2; 3]> {
    let vmaps = mesh.vmaps.as_deref()?;
    let uv = |corner: &EditableFaceCorner| -> Option<DVec2> {
      mesh
        .vmap_references
        .get(corner.vmap_reference as usize)?
        .points
        .iter()
        .find_map(|entry| {
          let vmap = vmaps.get(entry.vmap as usize)?;

          if vmap.kind != EditableVMapKind::Uv {
            return None;
          }

          match vmap.get_entry(entry.index as usize)? {
            [u, v] => Some(DVec2::new(f64::from(*u), f64::from(*v))),
            _ => None,
          }
        })
    };

    Some([uv(&corners[0])?, uv(&corners[1])?, uv(&corners[2])?])
  }

  fn intern(&mut self, texture: &str) -> u32 {
    let next: u32 = u32::try_from(self.textures.len()).unwrap_or(u32::MAX);

    *self.textures.entry(texture.to_lowercase()).or_insert(next)
  }
}
