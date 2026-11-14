use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjFile, BuildPrjMaterial};
use xrf_math::Aabb;

use crate::geometry::geometry_face::GeometryFace;
use crate::geometry::geometry_mesh::GeometryMesh;

impl GeometryMesh {
  /// The game material ids xrLC takes, below `1 << 16` (`R_ASSERT(B.dwMaterialGame<65536)`, `Build_Load.cpp:98`).
  const GAME_MATERIAL_LIMIT: u32 = 1 << 16;

  /// A material's compiler shader where it has none, as a glow's material has (`post_process_materials`).
  const NO_COMPILER_SHADER: u16 = u16::MAX;

  /// The static geometry of `project`, as `CBuild::Load` reads it (`xrLC/Build_Load.cpp:47`), checked first for what
  /// xrLC would stop on or read past: a face naming a vertex or material the project lacks, a game material past its
  /// limit, a material naming a texture or shader the project lacks, and smoothing flags missing or not one a face.
  /// The smoothing flags are the edge flags of a project from Clear Sky's editor on; Shadow of Chernobyl's has none,
  /// which xrLC reads only under `-nosmg`.
  ///
  /// # Errors
  ///
  /// Returns an error naming every problem found, up to a few of each kind with their counts.
  pub fn load(project: &BuildPrjFile) -> XrfResult<Self> {
    Self::check(project)?;

    let smoothing: &[u32] = project.smoothing_groups.as_deref().unwrap_or_default();
    let bounds: Aabb = Aabb::from_points(&project.vertices)
      .ok_or_else(|| XrfError::new_invalid_error("The project holds no static geometry to compile"))?;

    Ok(Self {
      vertices: project.vertices.clone(),
      faces: project
        .faces
        .iter()
        .zip(smoothing)
        .enumerate()
        .map(|(index, (face, smoothing))| GeometryFace {
          vertices: face.vertices,
          texture_coordinates: face.texture_coordinates,
          material: face.material,
          game_material: face.game_material,
          smoothing: *smoothing,
          source: index as u32,
        })
        .collect(),
      bounds,
    })
  }

  /// Every problem of `project` that would stop xrLC or have it read past a table.
  fn check(project: &BuildPrjFile) -> XrfResult {
    let mut problems: Vec<String> = Vec::new();
    let mut report = |kind: &str, found: Vec<String>| {
      if !found.is_empty() {
        let shown: Vec<String> = found.iter().take(5).cloned().collect();

        problems.push(format!("{} {kind}: {}", found.len(), shown.join("; ")));
      }
    };

    match &project.smoothing_groups {
      None => report(
        "faces without smoothing flags",
        vec![String::from("the project carries none")],
      ),
      Some(groups) if groups.len() != project.faces.len() => report(
        "smoothing flags not one a face",
        vec![format!("{} flags for {} faces", groups.len(), project.faces.len())],
      ),
      Some(_) => {}
    }

    let vertices: usize = project.vertices.len();
    let materials: &[BuildPrjMaterial] = &project.materials;

    report(
      "faces naming a vertex the project lacks",
      Self::find(&project.faces, |face| {
        face
          .vertices
          .iter()
          .find(|vertex| **vertex as usize >= vertices)
          .map(|vertex| format!("vertex {vertex} of {vertices}"))
      }),
    );
    report(
      "faces naming a material the project lacks",
      Self::find(&project.faces, |face| {
        (usize::from(face.material) >= materials.len())
          .then(|| format!("material {} of {}", face.material, materials.len()))
      }),
    );
    report(
      "faces of a material with no compiler shader, which xrLC would read past its library for",
      Self::find(&project.faces, |face| {
        materials
          .get(usize::from(face.material))
          .is_some_and(|material| material.compiler_shader == Self::NO_COMPILER_SHADER)
          .then(|| format!("material {}", face.material))
      }),
    );
    report(
      "faces of a game material past xrLC's limit",
      Self::find(&project.faces, |face| {
        (face.game_material >= Self::GAME_MATERIAL_LIMIT).then(|| format!("game material {}", face.game_material))
      }),
    );
    report(
      "materials naming a texture or shader the project lacks",
      Self::find(materials, |material| {
        (usize::from(material.texture) >= project.textures.len()
          || usize::from(material.shader) >= project.render_shaders.len()
          || (material.compiler_shader != Self::NO_COMPILER_SHADER
            && usize::from(material.compiler_shader) >= project.compiler_shaders.len()))
        .then(|| {
          format!(
            "texture {} of {}, shader {} of {}, compiler shader {} of {}",
            material.texture,
            project.textures.len(),
            material.shader,
            project.render_shaders.len(),
            material.compiler_shader,
            project.compiler_shaders.len()
          )
        })
      }),
    );

    if problems.is_empty() {
      Ok(())
    } else {
      Err(XrfError::new_invalid_error(format!(
        "The project cannot be compiled: {}",
        problems.join("; ")
      )))
    }
  }

  /// Each of `records` `problem` finds one in, by index.
  fn find<T>(records: &[T], problem: impl Fn(&T) -> Option<String>) -> Vec<String> {
    records
      .iter()
      .enumerate()
      .filter_map(|(index, record)| problem(record).map(|found| format!("#{index}: {found}")))
      .collect()
  }
}
