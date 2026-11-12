use std::collections::HashMap;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use glam::{DVec2, DVec3};
use xrf_editable_object::{EditableFaceCorner, EditableMesh, EditableObjectFile, EditableSurface, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};
use xrf_utils::format_path;

use crate::level::instance::level_library_object::{LevelLibraryObject, LevelLibraryTriangle};

/// The objects of object libraries a decompile looks for in a level: every static object under the roots with a surface
/// that renders (not dynamic, an occluder or a sound occluder, which the Build does not merge into the geometry), the
/// multi-use ones apart as the trees a level's tree models may be, and the textures their surfaces draw, each numbered
/// once.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelInstanceLibrary {
  /// The objects the Build merges into the static geometry, and the multi-use ones it builds once and references; each
  /// list the largest first.
  pub objects: Vec<LevelLibraryObject>,
  pub trees: Vec<LevelLibraryObject>,
  /// Textures by their lowercased name.
  textures: HashMap<String, u32>,
  /// Objects found under the roots that did not read.
  pub unread_objects: usize,
}

impl LevelInstanceLibrary {
  const OBJECT_EXTENSION: &'static str = "object";

  /// The `Shader_xrLC` flags of a compiler shader xrLC collides with and draws.
  const COLLISION_FLAG: u32 = ShaderCompilerShader::FLAGS[0].0;
  const RENDERING_FLAG: u32 = ShaderCompilerShader::FLAGS[1].0;

  /// Object flags that keep an object out of the level's geometry.
  const UNBUILT_FLAGS: u32 =
    EditableObjectFile::DYNAMIC_FLAG | EditableObjectFile::HOM_FLAG | EditableObjectFile::SOUND_OCCLUDER_FLAG;

  /// The objects under each of `roots`, searched through their folders, each surface's compiler shader looked up in
  /// `compiler_shaders` for whether xrLC draws it and collides with it.
  ///
  /// # Errors
  ///
  /// Returns an error when a root is not a directory that can be listed.
  pub fn read_from_roots<T: ByteOrder>(
    roots: &[PathBuf],
    compiler_shaders: Option<&ShaderCompilerFile>,
  ) -> XrfResult<Self> {
    let mut library: Self = Self::default();

    for root in roots {
      if !root.is_dir() {
        return Err(XrfError::new_not_found_error(format!(
          "Object library {} is not a directory",
          format_path(root)
        )));
      }

      library.read_directory::<T>(root, root, compiler_shaders)?;
    }

    for objects in [&mut library.objects, &mut library.trees] {
      objects.sort_by(|left, right| {
        right
          .area
          .total_cmp(&left.area)
          .then_with(|| left.reference.cmp(&right.reference))
      });
    }

    Ok(library)
  }

  /// The number `texture` takes in the table, any case; none where no library object draws it.
  pub fn get_texture(&self, texture: &str) -> Option<u32> {
    self.textures.get(&texture.to_lowercase()).copied()
  }

  /// Adds `object`, found at `path` and referenced as `reference`, where the Build builds it and it renders: to the
  /// trees where it is multi-use, else to the objects. Each surface's compiler shader is looked up in
  /// `compiler_shaders`, one it does not hold, or of none named, taken to draw and collide.
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

  /// `object` as the library would hold it, its textures numbered as the library numbers them and a surface of a
  /// texture no library object draws left out: a decompiled model to look for library objects in.
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
      compiler_shaders
        .and_then(|shaders| shaders.find_shader(name))
        .map_or(Self::COLLISION_FLAG | Self::RENDERING_FLAG, |shader| shader.flags)
    };

    for mesh in meshes {
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

        if surface_flags & Self::RENDERING_FLAG == 0 {
          if surface_flags & Self::COLLISION_FLAG != 0 {
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

  fn read_directory<T: ByteOrder>(
    &mut self,
    root: &Path,
    directory: &Path,
    compiler_shaders: Option<&ShaderCompilerFile>,
  ) -> XrfResult {
    let entries = std::fs::read_dir(directory)
      .map_err(|error| XrfError::new_read_error(format!("{} was not listed: {error}", format_path(directory))))?;
    let mut paths: Vec<PathBuf> = entries
      .map(|entry| entry.map(|entry| entry.path()))
      .collect::<Result<_, _>>()
      .map_err(|error| XrfError::new_read_error(format!("{} was not listed: {error}", format_path(directory))))?;

    // Listed in name order, so the library numbers its textures the same on every machine.
    paths.sort();

    for path in paths {
      if path.is_dir() {
        self.read_directory::<T>(root, &path, compiler_shaders)?;
      } else if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::OBJECT_EXTENSION))
      {
        match EditableObjectFile::read_from_path::<T, _>(&path) {
          Ok(object) => {
            let reference: String = Self::get_reference(root, &path);

            self.add(reference, path, &object, compiler_shaders);
          }
          Err(_) => self.unread_objects += 1,
        }
      }
    }

    Ok(())
  }

  /// The reference a scene object names the object at `path` under `root` by: its folders and name, the extension
  /// left off, joined by backslashes.
  fn get_reference(root: &Path, path: &Path) -> String {
    path
      .strip_prefix(root)
      .unwrap_or(path)
      .with_extension("")
      .components()
      .map(|component| component.as_os_str().to_string_lossy().into_owned())
      .collect::<Vec<String>>()
      .join("\\")
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
