use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_math::Vector3d;
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};
use xrf_utils::format_path;

use crate::level::level_collision_index::{LevelCollisionIndex, LevelCollisionLookup};
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;

/// How a level's decompile dresses its surfaces: from what the level shows, or with the editor's defaults.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSurfaces {
  /// Each face's game material from the collision face it lies on, and a compiler shader fitting what the level shows.
  #[default]
  Infer,
  /// Every surface takes `default` for both, as a new surface in the Actor Editor does.
  Defaults,
}

/// How often GSC, or whoever authored an object library, paired each engine shader with each compiler shader, counted
/// over the surfaces of every `.object` under the library's roots: what breaks a tie between compiler shaders that fit
/// what a compiled level shows of a surface.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelShaderPairings {
  /// Surfaces by engine shader, then by compiler shader, both lowercased.
  counts: HashMap<String, HashMap<String, usize>>,
  /// Objects found under the roots that did not read, and so did not count.
  pub unread_objects: usize,
}

impl LevelShaderPairings {
  const OBJECT_EXTENSION: &'static str = "object";

  /// The pairings of every `.object` under each of `roots`, searched through their folders.
  ///
  /// # Errors
  ///
  /// Returns an error when a root is not a directory that can be listed.
  pub fn read_from_roots<T: ByteOrder>(roots: &[PathBuf]) -> XrfResult<Self> {
    let mut pairings: Self = Self::default();

    for root in roots {
      if !root.is_dir() {
        return Err(XrfError::new_not_found_error(format!(
          "Object library {} is not a directory",
          format_path(root)
        )));
      }

      pairings.read_directory::<T>(root)?;
    }

    Ok(pairings)
  }

  fn read_directory<T: ByteOrder>(&mut self, directory: &Path) -> XrfResult {
    let entries = std::fs::read_dir(directory)
      .map_err(|error| XrfError::new_read_error(format!("{} was not listed: {error}", format_path(directory))))?;

    for entry in entries {
      let path: PathBuf = entry
        .map_err(|error| XrfError::new_read_error(format!("{} was not listed: {error}", format_path(directory))))?
        .path();

      if path.is_dir() {
        self.read_directory::<T>(&path)?;
      } else if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::OBJECT_EXTENSION))
      {
        match EditableObjectFile::read_from_path::<T, _>(&path) {
          Ok(object) => {
            for surface in object.surfaces.unwrap_or_default() {
              self.add(&surface.shader, &surface.compiler_shader);
            }
          }
          Err(_) => self.unread_objects += 1,
        }
      }
    }

    Ok(())
  }

  /// Counts one surface of `engine` shader paired with `compiler` shader.
  pub fn add(&mut self, engine: &str, compiler: &str) {
    *self
      .counts
      .entry(engine.to_lowercase())
      .or_default()
      .entry(compiler.to_lowercase())
      .or_default() += 1;
  }

  /// How many surfaces paired `engine` with `compiler`.
  pub fn get(&self, engine: &str, compiler: &str) -> usize {
    self
      .counts
      .get(&engine.to_lowercase())
      .and_then(|compilers| compilers.get(&compiler.to_lowercase()))
      .copied()
      .unwrap_or(0)
  }
}

/// Why a compiler shader was chosen among those fitting a surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelCompilerShaderReason {
  /// The library's objects pair the surface's engine shader with it most often.
  Paired,
  /// No library object pairs the engine shader with any that fits, and it is the one GSC's objects use most for what
  /// the level shows.
  CommonestFit,
  /// The first in `shaders_xrlc.xr` that fits.
  FirstFit,
  /// None fits, so the editor's `default`.
  NoFit,
}

/// The compiler shader a decompile chose for the surfaces of one engine shader showing one kind of evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompilerShaderChoice {
  pub engine_shader: String,
  /// Whether its faces lie on collision faces.
  pub collision: bool,
  /// Whether its vertices store a baked colour in place of lightmap coordinates.
  pub vertex_lighting: bool,
  pub compiler_shader: String,
  pub reason: LevelCompilerShaderReason,
  /// How many surfaces the library pairs the engine shader with it, where it does.
  pub pairings: usize,
}

/// What the surfaces of a level's decompile were dressed from, counted.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSurfaceCensus {
  /// Triangles lying on a collision face of the very same three points.
  pub exact_triangles: usize,
  /// Triangles a ray from just above found a collision face beneath, as the level compiler splits render faces it
  /// does not split in the collision form.
  pub traced_triangles: usize,
  /// Triangles on no collision face, which the level compiler built from a compiler shader of no collision.
  pub uncollided_triangles: usize,
  /// Collision materials the game material library does not define, by number; their faces take `default`.
  pub unknown_game_materials: BTreeSet<u16>,
  pub compiler_shaders: Vec<LevelCompilerShaderChoice>,
}

/// What dresses a level's surfaces from where their triangles lie on its collision form
/// ([`LevelCollisionIndex`]): the libraries naming game materials and compiler shaders, and an object library's
/// pairings.
pub struct LevelSurfaceInference {
  game_materials: GameMtlFile,
  compiler_shaders: ShaderCompilerFile,
  pairings: LevelShaderPairings,
  /// Each compiler shader chosen, by engine shader, collision and vertex lighting.
  choices: BTreeMap<(String, bool, bool), LevelCompilerShaderChoice>,
  census: LevelSurfaceCensus,
}

impl LevelSurfaceInference {
  /// The libraries inferring surfaces reads, by the names a game's `gamedata` holds them under.
  pub const GAME_MATERIALS_FILE: &'static str = GameMtlFile::FILE_NAME;
  pub const COMPILER_SHADERS_FILE: &'static str = ShaderCompilerFile::FILE_NAME;

  /// The compiler shaders GSC's objects use most for render geometry, by collision and vertex lighting: measured over
  /// the CoP SDK 0.7's 4,983 surfaces, where with the engine shader's own pairing first they give back GSC's choice for
  /// 95.5%, and alone for 84.2%.
  const COMMONEST: [((bool, bool), &'static str); 4] = [
    ((true, false), "default"),
    ((true, true), "def_shaders\\def_vertex"),
    ((false, false), "def_shaders\\def_ghost"),
    ((false, true), "def_shaders\\def_vertex_ghost"),
  ];

  /// `Shader_xrLC` flags: collision, rendering, vertex lighting.
  const COLLISION_FLAG: u32 = ShaderCompilerShader::FLAGS[0].0;
  const RENDERING_FLAG: u32 = ShaderCompilerShader::FLAGS[1].0;
  const VERTEX_LIGHTING_FLAG: u32 = ShaderCompilerShader::FLAGS[3].0;

  pub fn new(game_materials: GameMtlFile, compiler_shaders: ShaderCompilerFile, pairings: LevelShaderPairings) -> Self {
    Self {
      game_materials,
      compiler_shaders,
      pairings,
      choices: BTreeMap::new(),
      census: LevelSurfaceCensus::default(),
    }
  }

  /// The census so far, its compiler shader choices in a fixed order.
  pub fn into_census(self) -> LevelSurfaceCensus {
    LevelSurfaceCensus {
      compiler_shaders: self.choices.into_values().collect(),
      ..self.census
    }
  }

  /// `geometry` split by the game material each of its triangles lies on, each part dressed with that material and a
  /// compiler shader fitting it: collision where it lies on one, vertex lighting where its vertices keep no lightmap
  /// coordinates.
  pub fn dress(
    &mut self,
    collision: &mut LevelCollisionIndex,
    geometry: LevelVisualGeometry,
  ) -> Vec<LevelVisualGeometry> {
    let vertex_lighting: bool = geometry
      .vertices
      .first()
      .is_some_and(|vertex| vertex.lightmap_coordinate.is_none());
    let mut parts: BTreeMap<Option<u16>, Vec<u16>> = BTreeMap::new();

    for triangle in geometry.indices.as_chunks::<3>().0 {
      let points: Option<[Vector3d; 3]> = triangle
        .iter()
        .map(|index| geometry.vertices.get(usize::from(*index)).map(|vertex| vertex.position))
        .collect::<Option<Vec<Vector3d>>>()
        .and_then(|points| points.try_into().ok());
      let lookup: LevelCollisionLookup =
        points.map_or(LevelCollisionLookup::None, |points| collision.find_static(&points));

      match lookup {
        LevelCollisionLookup::Exact(_) => self.census.exact_triangles += 1,
        LevelCollisionLookup::Traced(_) => self.census.traced_triangles += 1,
        LevelCollisionLookup::None => self.census.uncollided_triangles += 1,
      }

      let material: Option<u16> = lookup.get_material();

      parts.entry(material).or_default().extend_from_slice(triangle);
    }

    parts
      .into_iter()
      .map(|(material, indices)| LevelVisualGeometry {
        dress: SurfaceDress {
          compiler_shader: self.choose_compiler_shader(&geometry.shader, material.is_some(), vertex_lighting),
          game_material: self.get_material_name(material),
        },
        texture: geometry.texture.clone(),
        shader: geometry.shader.clone(),
        vertices: geometry.vertices.clone(),
        indices,
      })
      .collect()
  }

  /// The game material library the surfaces are named from.
  pub fn get_game_materials(&self) -> &GameMtlFile {
    &self.game_materials
  }

  /// The game material numbered `material` by name, or `default` for no collision or a number the library lacks.
  pub fn get_material_name(&mut self, material: Option<u16>) -> String {
    let Some(material) = material else {
      return String::from(SurfaceDress::DEFAULT_SETTING);
    };

    match self.game_materials.find_material(u32::from(material)) {
      Some(found) => found.name.clone(),
      None => {
        self.census.unknown_game_materials.insert(material);
        String::from(SurfaceDress::DEFAULT_SETTING)
      }
    }
  }

  /// The compiler shader of a surface of `engine` shader showing `collision` and `vertex_lighting`: among those of
  /// `shaders_xrlc.xr` that render and fit both, the one the library pairs with the engine shader most, then the one
  /// GSC's objects use most for that evidence, then the first.
  fn choose_compiler_shader(&mut self, engine: &str, collision: bool, vertex_lighting: bool) -> String {
    let key: (String, bool, bool) = (engine.to_lowercase(), collision, vertex_lighting);

    if let Some(choice) = self.choices.get(&key) {
      return choice.compiler_shader.clone();
    }

    let wanted: u32 = Self::RENDERING_FLAG
      | if collision { Self::COLLISION_FLAG } else { 0 }
      | if vertex_lighting { Self::VERTEX_LIGHTING_FLAG } else { 0 };
    let mask: u32 = Self::COLLISION_FLAG | Self::RENDERING_FLAG | Self::VERTEX_LIGHTING_FLAG;
    let commonest: Option<&str> = Self::COMMONEST
      .iter()
      .find(|(evidence, _)| *evidence == (collision, vertex_lighting))
      .map(|(_, name)| *name);
    let best = self
      .compiler_shaders
      .shaders
      .iter()
      .enumerate()
      .filter(|(_, shader)| shader.flags & mask == wanted)
      .map(|(index, shader)| {
        let pairings: usize = self.pairings.get(engine, &shader.name);
        let is_commonest: bool = commonest.is_some_and(|name| shader.name.eq_ignore_ascii_case(name));

        ((pairings, is_commonest, std::cmp::Reverse(index)), shader)
      })
      .max_by_key(|(rank, _)| *rank);
    let choice: LevelCompilerShaderChoice = match best {
      Some(((pairings, is_commonest, _), shader)) => LevelCompilerShaderChoice {
        engine_shader: engine.to_owned(),
        collision,
        vertex_lighting,
        compiler_shader: shader.name.clone(),
        reason: if pairings > 0 {
          LevelCompilerShaderReason::Paired
        } else if is_commonest {
          LevelCompilerShaderReason::CommonestFit
        } else {
          LevelCompilerShaderReason::FirstFit
        },
        pairings,
      },
      None => LevelCompilerShaderChoice {
        engine_shader: engine.to_owned(),
        collision,
        vertex_lighting,
        compiler_shader: String::from(SurfaceDress::DEFAULT_SETTING),
        reason: LevelCompilerShaderReason::NoFit,
        pairings: 0,
      },
    };
    let name: String = choice.compiler_shader.clone();

    self.choices.insert(key, choice);

    name
  }
}
