use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelGeomPacker, LevelGeomRange, LevelHeaderChunk, LevelVertexDeclaration, LevelVertexLayout, LevelVertexQuantizer,
  LevelVertexStored,
};
use xrf_math::{Vector2d, Vector3d};

use crate::compiler_texture::CompilerTexture;
use crate::geometry::geometry_draft_light::GeometryDraftLight;
use crate::geometry::geometry_packed_visual::GeometryPackedVisual;
use crate::geometry::geometry_packed_visuals::GeometryPackedVisuals;
use crate::geometry::geometry_visual::GeometryVisual;
use crate::geometry::geometry_visual_vertex::GeometryVisualVertex;
use crate::geometry::geometry_visuals_request::GeometryVisualsRequest;

/// A level's static visuals, a visual a split, as xrLC converts them (`Flex2OGF`, `xrLC/xrFlex2OGF.cpp:64`).
///
/// A visual takes its first face's material: its sector, its base texture, and its lightmaps, `lmap#N_1` and `_2`, or
/// for a texture lit implicitly the base's `_lm`; a vertex-lit visual has none. Each face's corners carry the split
/// vertex's place and normal, the corner's basis and base coordinate, and a lightmap coordinate: the base coordinate
/// for an implicitly lit texture, the draft lightmap's for the rest ([`GeometryDraftLight`]). The faces are welded in
/// order ([`GeometryVisual::add_face`]), the fast path taking an opaque texture's, and a visual whose compiler shader
/// optimizes coordinates has them shifted ([`GeometryVisual::shift_texture_coordinates`]). Progressive meshes are left
/// to a later stage; the faces keep the order they were taken in, where xrLC strips them.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryVisuals {
  pub visuals: Vec<GeometryVisual>,
}

impl GeometryVisuals {
  /// The visuals of the request's splits.
  ///
  /// # Errors
  ///
  /// Returns an error for a material, texture or render shader the project or its textures lack, or a material of no
  /// compiler shader.
  pub fn build(request: &GeometryVisualsRequest<'_>) -> XrfResult<Self> {
    let GeometryVisualsRequest {
      mesh,
      normals,
      coordinates,
      tangents,
      splits,
      project,
      shaders,
      textures,
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
      let is_vertex_lit: bool = shader.is_vertex_lit();
      let mut names: Vec<String> = vec![texture.name.clone()];
      let mut texel: Vec<Vector2d> = vec![Self::get_texel(texture.width, texture.height)];

      if texture.is_implicitly_lit {
        names.push(format!("{}_lm.dds", texture.name));
        texel.push(texel[0]);
      } else if !is_vertex_lit {
        names.extend(GeometryDraftLight::LIGHTMAP_TEXTURES.map(String::from));
        texel.push(Self::get_texel(
          GeometryDraftLight::LIGHTMAP_SIDE,
          GeometryDraftLight::LIGHTMAP_SIDE,
        ));
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
        is_vertex_lit,
      );

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
            } else if is_vertex_lit {
              None
            } else {
              Some(GeometryDraftLight::LIGHTMAP_COORDINATE)
            },
          }
        });

        visual.add_face(corners, &texel, texture.is_opaque());
      }

      if shader.optimizes_uv() {
        visual.shift_texture_coordinates();
      }

      visuals.push(visual);
    }

    Ok(Self { visuals })
  }

  /// The visuals' vertices and indices packed into the level's buffers as `SaveTREE` packs them
  /// (`xrLC/xrSaveOGF.cpp:84`): in the order of their base textures' names, a stable sort, each visual's fast path
  /// before its own geometry; and their shader table, in visual order, each entry once.
  ///
  /// # Errors
  ///
  /// Returns an error when a visual's vertices cannot be packed.
  pub fn pack(&self, header: &LevelHeaderChunk) -> XrfResult<GeometryPackedVisuals> {
    let mut order: Vec<usize> = (0..self.visuals.len()).collect();

    order.sort_by(|first, second| self.visuals[*first].textures[0].cmp(&self.visuals[*second].textures[0]));

    let lightmapped: LevelVertexLayout = LevelVertexLayout::of_declaration(&LevelVertexDeclaration::get_lightmapped())?;
    let vertex_lit: LevelVertexLayout = LevelVertexLayout::of_declaration(&LevelVertexDeclaration::get_vertex_lit())?;
    let fast_path: LevelVertexLayout = LevelVertexLayout::of_declaration(&LevelVertexDeclaration::get_fast_path())?;
    let mut geometry: LevelGeomPacker = LevelGeomPacker::default();
    let mut fast: LevelGeomPacker = LevelGeomPacker::default();
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
        fast_indices = Some(fast.add_indices(visual.fast_faces.as_flattened()));
      }

      let (declaration, layout) = if visual.is_vertex_lit {
        (LevelVertexDeclaration::get_vertex_lit(), &vertex_lit)
      } else {
        (LevelVertexDeclaration::get_lightmapped(), &lightmapped)
      };
      let mut bytes: Vec<u8> = vec![0; visual.vertices.len() * layout.get_stride() as usize];

      for (vertex, slot) in visual
        .vertices
        .iter()
        .zip(bytes.chunks_exact_mut(layout.get_stride() as usize))
      {
        Self::get_stored(vertex, visual.is_vertex_lit).write_into::<XRayByteOrder>(layout, slot)?;
      }

      packed[index] = Some(GeometryPackedVisual {
        vertices: geometry.add_vertices(&declaration, &bytes)?,
        indices: geometry.add_indices(visual.faces.as_flattened()),
        fast_vertices,
        fast_indices,
      });
    }

    let mut shaders: Vec<String> = Vec::new();
    let mut shader_ids: Vec<u16> = Vec::with_capacity(self.visuals.len());

    for visual in &self.visuals {
      let id: usize = match shaders
        .iter()
        .position(|shader| shader.eq_ignore_ascii_case(&visual.shader))
      {
        Some(id) => id,
        None => {
          shaders.push(visual.shader.clone());
          shaders.len() - 1
        }
      };

      shader_ids.push(id as u16);
    }

    Ok(GeometryPackedVisuals {
      visuals: packed.into_iter().flatten().collect(),
      shader_ids,
      shaders,
      geometry: geometry.into_buffers(header.clone(), Vec::new()),
      fast: fast.into_buffers(header.clone(), Vec::new()),
    })
  }

  /// A texel of a texture of `width` by `height`, as a coordinate weld reaches.
  fn get_texel(width: u32, height: u32) -> Vector2d {
    Vector2d::new(1.0 / width as f32, 1.0 / height as f32)
  }

  /// A vertex's stored form under the draft light (`r1v_lmap` and `r1v_vert`).
  fn get_stored(vertex: &GeometryVisualVertex, is_vertex_lit: bool) -> LevelVertexStored {
    let length_squared: f32 =
      vertex.normal.x * vertex.normal.x + vertex.normal.y * vertex.normal.y + vertex.normal.z * vertex.normal.z;
    let scale: f32 = (1.0 / length_squared).sqrt();
    let normal: Vector3d = Vector3d::new(
      vertex.normal.x * scale,
      vertex.normal.y * scale,
      vertex.normal.z * scale,
    );
    let u = LevelVertexQuantizer::get_base_coordinate(vertex.texture_coordinate.x);
    let v = LevelVertexQuantizer::get_base_coordinate(vertex.texture_coordinate.y);
    let hemisphere: u8 = LevelVertexQuantizer::get_light_term(GeometryDraftLight::HEMISPHERE);
    let basis = |direction: [u8; 3], extra: u8| [direction[2], direction[1], direction[0], extra];

    LevelVertexStored {
      position: vertex.position,
      normal: Some(LevelVertexQuantizer::get_direction(&normal, hemisphere)),
      tangent: Some(basis(vertex.basis.tangent, u.fraction)),
      binormal: Some(basis(vertex.basis.binormal, v.fraction)),
      color: is_vertex_lit.then(|| {
        let rgb: u8 = LevelVertexQuantizer::get_light_term(GeometryDraftLight::STATIC_LIGHT);

        [
          rgb,
          rgb,
          rgb,
          LevelVertexQuantizer::get_light_term(GeometryDraftLight::SUN),
        ]
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
