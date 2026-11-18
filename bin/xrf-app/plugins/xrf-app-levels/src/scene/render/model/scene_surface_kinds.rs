use std::collections::HashMap;

use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableObjectFile, EditableSurface};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};
use xrf_vfs::XraySearch;

use crate::scene::{SceneObjectParts, SceneSurfaceKind};

/// Tells each surface of a scene's library objects its [`SceneSurfaceKind`]: not drawn by its compiler shader, dressed
/// in an editor marker texture, else drawn.
pub(crate) struct SceneSurfaceKinds {
  /// The game data's compiler shader library; none where it holds none that reads, so no surface is a collision one.
  library: Option<ShaderCompilerFile>,
}

impl SceneSurfaceKinds {
  /// The prefix of the SDK's marker textures, GSC's placeholders for what a player never sees (`prop_fake_nosun`,
  /// `prop_fake_occ`, `prop_fake_sound`, `prop_fake_kollision`, `prop_fake_ladder`, `prop_fake_dead`,
  /// `prop_fake_bed_fireplace`), which no real game surface uses.
  const EDITOR_TEXTURE_PREFIX: &'static str = "prop\\prop_fake_";

  /// Surfaces told apart by `library`'s compiler shaders.
  pub(crate) fn new(library: Option<ShaderCompilerFile>) -> Self {
    Self { library }
  }

  /// Surfaces told apart by the compiler shader library in `game`; with none, or one that does not read (logged), no
  /// surface is a collision one.
  pub(crate) fn open(game: &XraySearch) -> Self {
    let library: Option<ShaderCompilerFile> = match game.find(ShaderCompilerFile::FILE_NAME) {
      Ok(Some(_)) => game
        .read(ShaderCompilerFile::FILE_NAME)
        .and_then(ShaderCompilerFile::read_from_bytes::<XRayByteOrder>)
        .inspect_err(|error| {
          log::warn!("The scene's compiler shaders do not read, so every surface is drawn: {error}");
        })
        .ok(),
      Ok(None) => None,
      Err(error) => {
        log::warn!("The scene's compiler shaders cannot be looked for, so every surface is drawn: {error}");
        None
      }
    };

    Self::new(library)
  }

  /// The kind of `surface`: collision where its compiler shader, found in the library, renders nothing; else
  /// editor-only where its texture is a marker; else drawn, as is anything that cannot be told.
  pub(crate) fn get_kind(&self, surface: &EditableSurface) -> SceneSurfaceKind {
    let is_collision: bool = self
      .library
      .as_ref()
      .and_then(|library| library.find_shader(&surface.compiler_shader))
      .is_some_and(|shader: &ShaderCompilerShader| !shader.is_rendered());
    let is_marked: bool = surface
      .texture
      .get(..Self::EDITOR_TEXTURE_PREFIX.len())
      .is_some_and(|prefix| prefix.eq_ignore_ascii_case(Self::EDITOR_TEXTURE_PREFIX));

    if is_collision {
      SceneSurfaceKind::Collision
    } else if is_marked {
      SceneSurfaceKind::EditorOnly
    } else {
      SceneSurfaceKind::Drawn
    }
  }

  /// Splits `object` into a part per asked kind it has faces of, each the object without the other kinds' faces so it
  /// compiles as the object would; an object of one kind alone is that part whole.
  pub(crate) fn split(&self, object: EditableObjectFile, kinds: &[SceneSurfaceKind]) -> SceneObjectParts {
    // Owned, since the last part takes the object they would borrow.
    let surface_kinds: HashMap<String, SceneSurfaceKind> = object
      .surfaces
      .iter()
      .flatten()
      .map(|surface| (surface.name.clone(), self.get_kind(surface)))
      .collect();
    let kind_of = |name: &str| surface_kinds.get(name).copied().unwrap_or(SceneSurfaceKind::Drawn);
    let held: Vec<SceneSurfaceKind> = SceneSurfaceKind::ALL
      .into_iter()
      .filter(|kind| Self::has_faces(&object, |name| kind_of(name) == *kind))
      .collect();
    let mut parts: SceneObjectParts = SceneObjectParts::default();

    match held.as_slice() {
      [] => parts.insert(SceneSurfaceKind::Drawn, object),
      [kind] => parts.insert(*kind, object),
      _ => {
        let asked: Vec<SceneSurfaceKind> = held.into_iter().filter(|kind| kinds.contains(kind)).collect();

        // The last part takes the object itself, so a large shell is held at most twice while it is split.
        if let Some((last, rest)) = asked.split_last() {
          for kind in rest {
            parts.insert(*kind, Self::keep(object.clone(), |name| kind_of(name) == *kind));
          }

          parts.insert(*last, Self::keep(object, |name| kind_of(name) == *last));
        }
      }
    }

    parts
  }

  /// `part` with only the faces of the surfaces `is_kept` names.
  fn keep(mut part: EditableObjectFile, is_kept: impl Fn(&str) -> bool) -> EditableObjectFile {
    for mesh in part.meshes.iter_mut().flatten() {
      mesh
        .surface_faces
        .retain(|surface_faces| is_kept(&surface_faces.surface));
    }

    part
  }

  /// Whether a mesh of `object` holds a face of a surface `is_counted` names.
  fn has_faces(object: &EditableObjectFile, is_counted: impl Fn(&str) -> bool) -> bool {
    object.meshes.iter().flatten().any(|mesh| {
      mesh
        .surface_faces
        .iter()
        .any(|surface_faces| !surface_faces.faces.is_empty() && is_counted(&surface_faces.surface))
    })
  }
}
