use rayon::prelude::*;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelGeomPacker, LevelGeomRange, LevelVertexDeclaration, LevelVertexLayout, LevelVertexQuantizer, LevelVertexStored,
};
use xrf_math::{Vector2d, Vector3d};

use crate::geometry::level::geometry_packed_visual::GeometryPackedVisual;
use crate::geometry::visual::geometry_fast_path::GeometryFastPath;
use crate::geometry::visual::geometry_visual::GeometryVisual;
use crate::geometry::visual::geometry_visual_light::GeometryVisualLight;
use crate::geometry::visual::geometry_visual_vertex::GeometryVisualVertex;
use crate::geometry::visual::geometry_visuals_request::GeometryVisualsRequest;
use crate::input::compiler_texture::CompilerTexture;
use crate::light::light_draft::LightDraft;
use crate::light::lightmap::atlas::lightmap_faces::LightmapFaces;
use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::point::light_color::LightColor;

/// A level's static visuals, one a split, as `Flex2OGF` converts them.
///
/// A visual takes its first face's material, sector, base texture and lightmaps: that face's page's `lmap#N_1` and
/// `_2`, the base's `_lm` for an implicitly lit texture, none for a vertex-lit one. A corner's lightmap coordinate is
/// its base coordinate for an implicitly lit texture, else its chart's, or the draft lightmap's (`LightDraft`) without
/// a bake. Faces keep the order they were taken in, where xrLC strips them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeometryVisuals {
  pub visuals: Vec<GeometryVisual>,
}

impl GeometryVisuals {
  /// The visuals of the request's splits.
  ///
  /// # Errors
  ///
  /// Returns an error for a missing material, texture or render shader, a material of no compiler shader, or a split
  /// welding into more vertices than 16-bit indices reach.
  pub(crate) fn build(request: &GeometryVisualsRequest<'_>) -> XrfResult<Self> {
    let GeometryVisualsRequest {
      mesh,
      normals,
      coordinates,
      tangents,
      splits,
      project,
      shaders,
      textures,
      lights,
      lightmaps,
    } = *request;
    let mut visuals: Vec<GeometryVisual> = Vec::with_capacity(splits.splits.len());

    for split in &splits.splits {
      let Some(first) = split.first() else {
        continue;
      };
      let material_index: u16 = mesh.faces[*first as usize].material;
      let material = project.materials.get(usize::from(material_index)).ok_or_else(|| {
        XrfError::new_invalid_error(format!("A face names material {material_index}, which build.prj lacks"))
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
      let shader = shaders.get(material_index).ok_or_else(|| {
        XrfError::new_invalid_error(format!("build.prj material {material_index} has no compiler shader"))
      })?;
      let light: GeometryVisualLight = if shader.is_vertex_lit() {
        GeometryVisualLight::Vertex
      } else {
        GeometryVisualLight::Lightmap
      };
      let mut names: Vec<String> = vec![texture.name.clone()];
      let mut texel: Vec<Vector2d> = vec![Self::get_texel(texture.width, texture.height)];
      let page: Option<u32> = lightmaps.and_then(|lightmaps| lightmaps.pages[*first as usize]);

      if texture.is_implicitly_lit {
        names.push(format!("{}_lm.dds", texture.name));
        texel.push(texel[0]);
      } else if light == GeometryVisualLight::Lightmap {
        match page {
          Some(page) => {
            names.extend(LightmapFaces::get_textures(page));
            texel.push(Self::get_texel(LightChart::PAGE_SIDE, LightChart::PAGE_SIDE));
          }
          None => {
            names.extend(LightmapFaces::get_textures(LightDraft::LIGHTMAP_PAGE));
            texel.push(Self::get_texel(LightDraft::LIGHTMAP_SIDE, LightDraft::LIGHTMAP_SIDE));
          }
        }
      }

      let dress: String = names
        .iter()
        .map(|name| name.split('.').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",");
      let mut visual: GeometryVisual = GeometryVisual::new(
        material_index,
        material.sector,
        names,
        format!("{}/{dress}", render_shader.name.value),
        light,
      );
      let fast_path: GeometryFastPath = if texture.is_opaque() {
        GeometryFastPath::Joined
      } else {
        GeometryFastPath::Left
      };

      for face in split {
        let face: usize = *face as usize;
        let corners: [GeometryVisualVertex; 3] = std::array::from_fn(|corner| {
          let split_vertex: usize = normals.faces[face][corner] as usize;
          let texture_coordinate: Vector2d = coordinates.faces[face][corner];

          GeometryVisualVertex {
            position: normals.positions[split_vertex],
            normal: normals.normals[split_vertex],
            basis: tangents.faces[face][corner],
            texture_coordinate,
            lightmap_coordinate: if texture.is_implicitly_lit {
              Some(texture_coordinate)
            } else if light == GeometryVisualLight::Vertex {
              None
            } else if let (Some(lightmaps), Some(_)) = (lightmaps, page) {
              Some(lightmaps.coordinates[face][corner])
            } else {
              Some(LightDraft::LIGHTMAP_COORDINATE)
            },
            light: lights.map_or(LightDraft::VERTEX, |lights| lights[split_vertex]),
          }
        });

        visual.add_face(corners, &texel, fast_path)?;
      }

      if shader.optimizes_uv() {
        visual.shift_texture_coordinates();
      }

      visuals.push(visual);
    }

    // `Flex2OGF` makes each visual progressive, except at draft quality (`OGF::MakeProgressive`).
    if !project.params.is_draft() {
      visuals
        .par_iter_mut()
        .for_each(|visual| visual.make_progressive(GeometryVisual::STATIC_METRIC_LIMIT));
    }

    Ok(Self { visuals })
  }

  /// Packs the visuals into `geometry` (`level.geom`) and `fast` (`level.geomx`) as `SaveTREE` does: stably sorted by
  /// base texture name, each fast path before its visual's geometry. Returns where each went, in visual order.
  ///
  /// # Errors
  ///
  /// Returns an error when a visual's vertices cannot be packed.
  pub(crate) fn pack(
    &self,
    geometry: &mut LevelGeomPacker,
    fast: &mut LevelGeomPacker,
  ) -> XrfResult<Vec<GeometryPackedVisual>> {
    let mut order: Vec<usize> = (0..self.visuals.len()).collect();

    order.sort_by(|first, second| self.visuals[*first].textures[0].cmp(&self.visuals[*second].textures[0]));

    let lightmapped: LevelVertexLayout = LevelVertexLayout::of_declaration(&LevelVertexDeclaration::get_lightmapped())?;
    let vertex_lit: LevelVertexLayout = LevelVertexLayout::of_declaration(&LevelVertexDeclaration::get_vertex_lit())?;
    let fast_path: LevelVertexLayout = LevelVertexLayout::of_declaration(&LevelVertexDeclaration::get_fast_path())?;
    let mut packed: Vec<Option<GeometryPackedVisual>> = vec![None; self.visuals.len()];

    for index in order {
      let visual: &GeometryVisual = &self.visuals[index];
      let (mut fast_vertices, mut fast_indices): (Option<LevelGeomRange>, Option<LevelGeomRange>) = (None, None);

      if !visual.fast_vertices.is_empty() && !visual.fast_faces.is_empty() {
        let mut bytes: Vec<u8> = vec![0; visual.fast_vertices.len() * fast_path.get_stride() as usize];

        for (vertex, slot) in visual
          .fast_vertices
          .iter()
          .zip(bytes.chunks_exact_mut(fast_path.get_stride() as usize))
        {
          Self::get_fast_stored(vertex).write_into::<XRayByteOrder>(&fast_path, slot)?;
        }

        fast_vertices = Some(fast.add_vertices(&LevelVertexDeclaration::get_fast_path(), &bytes)?);
        fast_indices = Some(fast.add_indices(visual.fast_faces.as_flattened())?);
      }

      let (declaration, layout) = match visual.light {
        GeometryVisualLight::Vertex => (LevelVertexDeclaration::get_vertex_lit(), &vertex_lit),
        GeometryVisualLight::Lightmap | GeometryVisualLight::Tree => {
          (LevelVertexDeclaration::get_lightmapped(), &lightmapped)
        }
      };
      let mut bytes: Vec<u8> = vec![0; visual.vertices.len() * layout.get_stride() as usize];

      for (vertex, slot) in visual
        .vertices
        .iter()
        .zip(bytes.chunks_exact_mut(layout.get_stride() as usize))
      {
        Self::get_stored(vertex, visual.light).write_into::<XRayByteOrder>(layout, slot)?;
      }

      packed[index] = Some(GeometryPackedVisual {
        vertices: geometry.add_vertices(&declaration, &bytes)?,
        indices: geometry.add_indices(visual.faces.as_flattened())?,
        fast_vertices,
        fast_indices,
        slide_window: None,
      });
    }

    Ok(packed.into_iter().flatten().collect())
  }

  /// One texel of a `width` by `height` texture, a coordinate weld's reach.
  fn get_texel(width: u32, height: u32) -> Vector2d {
    Vector2d::new(1.0 / width as f32, 1.0 / height as f32)
  }

  /// A vertex's stored form (`r1v_lmap` and `r1v_vert`): the sky term in the normal's fourth byte, then a vertex-lit
  /// one's static light and sun as its colour, or a lightmapped one's lightmap coordinates.
  fn get_stored(vertex: &GeometryVisualVertex, light: GeometryVisualLight) -> LevelVertexStored {
    let is_vertex_lit: bool = light == GeometryVisualLight::Vertex;
    let normal: Vector3d = vertex.normal.normalize();
    let u = LevelVertexQuantizer::get_base_coordinate(vertex.texture_coordinate.x);
    let v = LevelVertexQuantizer::get_base_coordinate(vertex.texture_coordinate.y);
    let term = LevelVertexQuantizer::get_light_term;
    let hemisphere: u8 = term(vertex.light.hemi);
    let basis = |direction: [u8; 3], extra: u8| [direction[2], direction[1], direction[0], extra];

    LevelVertexStored {
      position: vertex.position,
      normal: Some(LevelVertexQuantizer::get_direction(&normal, hemisphere)),
      tangent: Some(basis(vertex.basis.tangent, u.fraction)),
      binormal: Some(basis(vertex.basis.binormal, v.fraction)),
      color: is_vertex_lit.then(|| {
        let LightColor { rgb, sun, .. } = vertex.light;

        [term(rgb.z), term(rgb.y), term(rgb.x), term(sun)]
      }),
      texture_coordinate: Some([u.primary, v.primary, 0, 0]),
      lightmap_coordinate: vertex.lightmap_coordinate.filter(|_| !is_vertex_lit).map(|coordinate| {
        [
          LevelVertexQuantizer::get_lightmap_coordinate(coordinate.x),
          LevelVertexQuantizer::get_lightmap_coordinate(coordinate.y),
        ]
      }),
    }
  }

  /// A fast path vertex's stored form, its position alone.
  fn get_fast_stored(position: &Vector3d) -> LevelVertexStored {
    LevelVertexStored {
      position: *position,
      normal: None,
      tangent: None,
      binormal: None,
      color: None,
      texture_coordinate: None,
      lightmap_coordinate: None,
    }
  }
}
