use std::collections::BTreeMap;

use xrf_gamemtl::GameMtlFile;
use xrf_math::Vector3d;
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::level::geometry::level_collision_index::LevelCollisionIndex;
use crate::level::geometry::level_collision_lookup::LevelCollisionLookup;
use crate::level::geometry::level_visual_geometry::LevelVisualGeometry;
use crate::level::surfaces::level_compiler_shader_choice::LevelCompilerShaderChoice;
use crate::level::surfaces::level_compiler_shader_reason::LevelCompilerShaderReason;
use crate::level::surfaces::level_shader_pairings::LevelShaderPairings;
use crate::level::surfaces::level_surface_census::LevelSurfaceCensus;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;

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

    let wanted: u32 = ShaderCompilerShader::RENDERING
      | if collision { ShaderCompilerShader::COLLISION } else { 0 }
      | if vertex_lighting {
        ShaderCompilerShader::VERTEX_LIGHTING
      } else {
        0
      };
    let mask: u32 =
      ShaderCompilerShader::COLLISION | ShaderCompilerShader::RENDERING | ShaderCompilerShader::VERTEX_LIGHTING;
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
