use rayon::prelude::*;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  BuildPrjFile, BuildPrjLod, BuildPrjMuModel, LevelGeomPacker, LevelGeomSlideWindow, LevelGeomSlideWindowItem,
  LevelVertexDeclaration, LevelVertexLayout, LevelVertexQuantizer, LevelVertexStored,
};
use xrf_math::{EPS, Vector2d, Vector3d};

use crate::compiler_shaders::CompilerShaders;
use crate::compiler_texture::CompilerTexture;
use crate::geometry::geometry_basis::GeometryBasis;
use crate::geometry::geometry_draft_light::GeometryDraftLight;
use crate::geometry::geometry_lod::GeometryLod;
use crate::geometry::geometry_lod_vertex::GeometryLodVertex;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_model::GeometryModel;
use crate::geometry::geometry_normals::GeometryNormals;
use crate::geometry::geometry_packed_visual::GeometryPackedVisual;
use crate::geometry::geometry_placement::GeometryPlacement;
use crate::geometry::geometry_tree::GeometryTree;
use crate::geometry::geometry_visual::GeometryVisual;
use crate::geometry::geometry_visual_vertex::GeometryVisualVertex;

/// A level's multi-use models as xrLC exports them (`RunAfterLight`, `xrLC/Build.cpp:330`): each model's visuals once,
/// each reference's placements of them, and its LOD.
///
/// - **A model** gets its own normals, the static pass's over its vertices, none welded, by its smoothing flags. Its
///   faces are sorted by material (`calc_materials`; xrLC's sort is unstable, this one keeps face order within a
///   material), and each rendering material's run becomes a visual: its faces welded as a static visual's with no
///   basis yet and no fast path, its coordinates shifted where its shader optimizes them, then its basis mended
///   ([`GeometryVisual::mend_basis`]), and above draft quality made progressive where it gains enough
///   ([`GeometryVisual::make_progressive`]).
/// - **A reference** places each of its model's visuals as a tree ([`GeometryTree`]), lit by the draft light's scale
///   and bias. Where its model names a LOD, a [`GeometryLod`] node stands over them, its quads placed by the reference
///   and their light the model's vertices' weighted by `1 / (1 + 100 d²)` from each corner.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryModels {
  /// Each `build.prj` multi-use model, in its order.
  pub models: Vec<GeometryModel>,
  /// Each reference's placements, in reference order, a model's visuals in theirs.
  pub trees: Vec<GeometryTree>,
  /// Each LOD node, after the trees of its reference.
  pub lods: Vec<GeometryLod>,
}

impl GeometryModels {
  /// The models and placements of `project`.
  ///
  /// # Errors
  ///
  /// Returns an error for a model whose faces xrLC would read past, a face of a material with no compiler shader or
  /// one the project lacks, a reference naming a model the project lacks, or a model naming a LOD it lacks.
  pub fn build(project: &BuildPrjFile, shaders: &CompilerShaders, textures: &[CompilerTexture]) -> XrfResult<Self> {
    let models: Vec<GeometryModel> = project
      .mu_models
      .par_iter()
      .map(|model| Self::build_model(model, project, shaders, textures))
      .collect::<XrfResult<_>>()?;

    let mut trees: Vec<GeometryTree> = Vec::new();
    let mut lods: Vec<GeometryLod> = Vec::new();

    for (index, reference) in project.mu_references.iter().enumerate() {
      let model: &BuildPrjMuModel = GeometryPlacement::get_model(project, index, reference)?;
      let first: usize = trees.len();

      for visual in 0..models[reference.model as usize].visuals.len() {
        trees.push(GeometryTree {
          model: reference.model,
          visual: visual as u32,
          reference: index as u32,
          sector: reference.sector,
          transform: reference.transform,
          scale: GeometryDraftLight::TREE_SCALE,
          bias: GeometryDraftLight::TREE_BIAS,
        });
      }

      if model.lod == u16::MAX {
        continue;
      }

      let lod: &BuildPrjLod = project.lods.get(usize::from(model.lod)).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Multi-use model {} names LOD {} of {}",
          model.name,
          model.lod,
          project.lods.len()
        ))
      })?;
      let placed: Vec<Vector3d> = models[reference.model as usize]
        .positions
        .iter()
        .map(|position| GeometryPlacement::transform(&reference.transform, position))
        .collect();

      lods.push(GeometryLod {
        reference: index as u32,
        material: lod.material,
        sector: reference.sector,
        trees: (first as u32..trees.len() as u32).collect(),
        faces: std::array::from_fn(|face| {
          std::array::from_fn(|corner| {
            let position: Vector3d =
              GeometryPlacement::transform(&reference.transform, &lod.faces[face].vertices[corner]);
            let light: [f32; 5] = Self::get_lod_light(&position, &placed);
            let term = |value: f32| LevelVertexQuantizer::get_light_term(value);

            GeometryLodVertex {
              position,
              texture_coordinate: lod.faces[face].texture_coordinates[corner],
              light: [term(light[0]), term(light[1]), term(light[2]), term(light[3])],
              sun: term(light[4]),
            }
          })
        }),
      });
    }

    Ok(Self { models, trees, lods })
  }

  /// The models' visuals packed into the level's buffers in the tree layout, model by model, as `export_geometry`
  /// packs them before the static visuals, a progressive one's windows added to `slide_windows`. Where each one went,
  /// by model and visual.
  ///
  /// # Errors
  ///
  /// Returns an error when a visual's vertices cannot be packed.
  pub fn pack(
    &self,
    geometry: &mut LevelGeomPacker,
    slide_windows: &mut Vec<LevelGeomSlideWindowItem>,
  ) -> XrfResult<Vec<Vec<GeometryPackedVisual>>> {
    let declaration = LevelVertexDeclaration::get_tree();
    let layout: LevelVertexLayout = LevelVertexLayout::of_declaration(&declaration)?;
    let stride: usize = layout.get_stride() as usize;
    let hemisphere: u8 = LevelVertexQuantizer::get_light_term(GeometryDraftLight::TREE_HEMISPHERE);
    let mut packed: Vec<Vec<GeometryPackedVisual>> = Vec::with_capacity(self.models.len());

    for model in &self.models {
      let mut visuals: Vec<GeometryPackedVisual> = Vec::with_capacity(model.visuals.len());

      for visual in &model.visuals {
        let mut bytes: Vec<u8> = vec![0; visual.vertices.len() * stride];

        for (vertex, slot) in visual.vertices.iter().zip(bytes.chunks_exact_mut(stride)) {
          Self::get_stored(vertex, model, hemisphere).write_into::<XRayByteOrder>(&layout, slot)?;
        }

        let slide_window: Option<u32> = visual.windows.as_ref().map(|windows| {
          slide_windows.push(LevelGeomSlideWindowItem {
            reserved: [0; 4],
            windows: windows
              .iter()
              .map(|window| LevelGeomSlideWindow {
                offset: window.offset,
                triangles: window.triangles,
                vertices: window.vertices,
              })
              .collect(),
          });

          (slide_windows.len() - 1) as u32
        });

        visuals.push(GeometryPackedVisual {
          vertices: geometry.add_vertices(&declaration, &bytes)?,
          indices: geometry.add_indices(visual.faces.as_flattened()),
          fast_vertices: None,
          fast_indices: None,
          slide_window,
        });
      }

      packed.push(visuals);
    }

    Ok(packed)
  }

  /// One model's visuals.
  #[expect(
    clippy::manual_midpoint,
    reason = "`Fbox::getcenter` sums and halves in single precision, which `f32::midpoint` does not"
  )]
  fn build_model(
    model: &BuildPrjMuModel,
    project: &BuildPrjFile,
    shaders: &CompilerShaders,
    textures: &[CompilerTexture],
  ) -> XrfResult<GeometryModel> {
    let mesh: GeometryMesh = GeometryMesh::of_model(model)?;
    let normals: GeometryNormals = GeometryNormals::of(&mesh);
    let mut order: Vec<usize> = (0..mesh.faces.len()).collect();

    order.sort_by_key(|face| mesh.faces[*face].material);

    let mut visuals: Vec<GeometryVisual> = Vec::new();

    for run in order.chunk_by(|first, second| mesh.faces[*first].material == mesh.faces[*second].material) {
      let material_index: u16 = mesh.faces[run[0]].material;
      let shader = shaders.require(model, run[0], &model.faces[run[0]])?;

      if !shader.is_rendered() {
        continue;
      }

      let material = project.materials.get(usize::from(material_index)).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "A multi-use face names material {material_index}, which build.prj lacks"
        ))
      })?;
      let texture: &CompilerTexture = textures.get(usize::from(material.texture)).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "build.prj material {material_index} names texture {}, which it does not hold",
          material.texture
        ))
      })?;
      let render_shader = project
        .render_shaders
        .get(usize::from(material.shader))
        .ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "build.prj material {material_index} names render shader {}, which it does not hold",
            material.shader
          ))
        })?;
      let texel: [Vector2d; 1] = [Vector2d::new(1.0 / texture.width as f32, 1.0 / texture.height as f32)];
      let mut visual: GeometryVisual = GeometryVisual::new(
        material_index,
        0,
        vec![texture.name.clone()],
        format!("{}/{}", render_shader.name.value, texture.name),
        false,
      );

      for face in run {
        let corners: [GeometryVisualVertex; 3] = std::array::from_fn(|corner| {
          let split: usize = normals.faces[*face][corner] as usize;

          GeometryVisualVertex {
            position: normals.positions[split],
            normal: normals.normals[split],
            basis: GeometryBasis::NONE,
            texture_coordinate: mesh.faces[*face].texture_coordinates[corner],
            lightmap_coordinate: None,
          }
        });

        visual.add_face(corners, &texel, false);
      }

      if shader.optimizes_uv() {
        visual.shift_texture_coordinates();
      }

      visual.mend_basis()?;

      // Each model visual is made progressive, except at draft quality (`xrMU_Model::calc_ogf`).
      if !project.params.is_draft() {
        visual.make_progressive(GeometryVisual::MODEL_METRIC_LIMIT);
      }

      visuals.push(visual);
    }

    let mut low: Vector3d = normals.positions.first().copied().unwrap_or(Vector3d::ZERO);
    let mut high: Vector3d = low;

    for position in &normals.positions {
      low = Vector3d::new(low.x.min(position.x), low.y.min(position.y), low.z.min(position.z));
      high = Vector3d::new(high.x.max(position.x), high.y.max(position.y), high.z.max(position.z));
    }

    Ok(GeometryModel {
      visuals,
      wind_base: Vector3d::new((low.x + high.x) * 0.5, low.y, (low.z + high.z) * 0.5),
      wind_height: high.y - low.y,
      positions: normals.positions,
    })
  }

  /// A LOD corner's light: each of the model's placed vertices' draft light weighted by `1 / (1 + 100 d²)`, summed in
  /// order and divided by the weights and `EPS`.
  fn get_lod_light(corner: &Vector3d, vertices: &[Vector3d]) -> [f32; 5] {
    let own: [f32; 5] = [
      GeometryDraftLight::STATIC_LIGHT,
      GeometryDraftLight::STATIC_LIGHT,
      GeometryDraftLight::STATIC_LIGHT,
      GeometryDraftLight::TREE_HEMISPHERE,
      GeometryDraftLight::TREE_SUN,
    ];
    let mut sum: [f32; 5] = [0.0; 5];
    let mut weights: f32 = 0.0;

    for vertex in vertices {
      let distance: f32 = corner.distance(vertex);
      let weight: f32 = 1.0 / (1.0 + 100.0 * distance * distance);

      for (term, value) in sum.iter_mut().zip(own) {
        *term += value * weight;
      }

      weights += weight;
    }

    let scale: f32 = 1.0 / (weights + EPS);

    sum.map(|term| term * scale)
  }

  /// A tree vertex's stored form (`export_geometry`): its normal as computed, the model's hemisphere in its fourth byte,
  /// its basis, and its coordinate with the wind fraction at 2,048 a tile.
  #[expect(
    clippy::manual_midpoint,
    reason = "xrLC averages the two wind terms in single precision, which `f32::midpoint` does not"
  )]
  fn get_stored(vertex: &GeometryVisualVertex, model: &GeometryModel, hemisphere: u8) -> LevelVertexStored {
    let basis = |direction: [u8; 3]| [direction[2], direction[1], direction[0], 0];
    let height: f32 = (vertex.position.y - model.wind_base.y) / model.wind_height;
    let reach: f32 = vertex.position.distance(&model.wind_base) / model.wind_height;

    LevelVertexStored {
      position: vertex.position,
      normal: Some(LevelVertexQuantizer::get_direction(&vertex.normal, hemisphere)),
      tangent: Some(basis(vertex.basis.tangent)),
      binormal: Some(basis(vertex.basis.binormal)),
      color: None,
      texture_coordinate: Some([
        LevelVertexQuantizer::get_tree_coordinate(vertex.texture_coordinate.x),
        LevelVertexQuantizer::get_tree_coordinate(vertex.texture_coordinate.y),
        LevelVertexQuantizer::get_tree_coordinate((height + reach) / 2.0),
        0,
      ]),
      lightmap_coordinate: None,
    }
  }
}
