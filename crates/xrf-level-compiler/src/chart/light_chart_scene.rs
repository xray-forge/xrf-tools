use xrf_geometry::FaceArea;
use xrf_level::BuildPrjFile;
use xrf_math::Vector3d;

use crate::compiler_shaders::CompilerShaders;
use crate::compiler_texture::CompilerTexture;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_normals::GeometryNormals;

/// What a level's lightmap charts are cut, sized and masked by: each face's corners over the split vertices, normal
/// and area, each material's light density and whether it is lightmapped, and the bake's density and jitter.
pub struct LightChartScene<'a> {
  pub mesh: &'a GeometryMesh,
  pub normals: &'a GeometryNormals,
  /// Each face's normal, `Face::N` (`Face::CalcNormal`).
  pub face_normals: Vec<Vector3d>,
  /// Each face's area, `Face::CalcArea`.
  pub areas: Vec<f32>,
  /// Each material's compiler shader's lightmap density, `lm_density`; 0 for one with no compiler shader.
  pub densities: Vec<f32>,
  /// Whether each material's faces are rendered.
  pub is_rendered: Vec<bool>,
  /// Whether each material's faces are lightmapped: rendered, not vertex-lit, and not of a texture lit implicitly.
  pub is_lightmapped: Vec<bool>,
  /// Each material's sector, which orders the charts as they are packed.
  pub sectors: Vec<u16>,
  /// The lightmaps' texels a metre.
  pub pixels_per_meter: f32,
  /// The texel samples a lightmap texel takes (`Jitter_Select`: 1, 9, else 4), which cover its mask.
  pub jitter_samples: u32,
  /// The angle, in degrees, beyond which faces no longer share a smoothed normal.
  pub smooth_angle: f32,
}

impl<'a> LightChartScene<'a> {
  /// The scene of `mesh`, its split vertices `normals`, `project`'s materials and parameters as `shaders` and
  /// `textures` resolve them, lit at `pixels_per_meter` with `jitter_samples` a texel.
  pub fn of(
    mesh: &'a GeometryMesh,
    normals: &'a GeometryNormals,
    project: &BuildPrjFile,
    shaders: &CompilerShaders,
    textures: &[CompilerTexture],
    pixels_per_meter: f32,
    jitter_samples: u32,
  ) -> Self {
    let (face_normals, areas): (Vec<Vector3d>, Vec<f32>) = mesh
      .faces
      .iter()
      .map(|face| {
        let corners: [Vector3d; 3] = face.get_corners(&mesh.vertices);

        (GeometryNormals::get_face_normal(&corners), FaceArea::of(&corners))
      })
      .unzip();
    let materials = 0..u16::try_from(project.materials.len()).unwrap_or(u16::MAX);

    Self {
      mesh,
      normals,
      face_normals,
      areas,
      densities: materials
        .clone()
        .map(|material| shaders.get(material).map_or(0.0, |shader| shader.lightmap_density))
        .collect(),
      is_rendered: materials
        .clone()
        .map(|material| shaders.get(material).is_some_and(|shader| shader.is_rendered()))
        .collect(),
      is_lightmapped: materials
        .map(|material| {
          let is_implicitly_lit: bool = project
            .materials
            .get(usize::from(material))
            .and_then(|entry| textures.get(usize::from(entry.texture)))
            .is_some_and(|texture| texture.is_implicitly_lit);

          shaders
            .get(material)
            .is_some_and(|shader| shader.is_rendered() && !shader.is_vertex_lit())
            && !is_implicitly_lit
        })
        .collect(),
      sectors: project.materials.iter().map(|material| material.sector).collect(),
      pixels_per_meter,
      jitter_samples,
      smooth_angle: project.params.smooth_angle,
    }
  }

  /// Whether `material`'s faces take charts.
  pub fn is_charted(&self, material: u16) -> bool {
    self.is_lightmapped.get(usize::from(material)).copied().unwrap_or(false)
  }
}
