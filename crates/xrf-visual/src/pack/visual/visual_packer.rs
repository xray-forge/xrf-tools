use xrf_math::Vector3d;
use xrf_ogf::{OgfFile, OgfGeometry, OgfModelType, OgfSlideWindow, OgfVertex, OgfVertexData};

use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::bounds::visual_box::VisualBox;
use crate::data::visual::bounds::visual_sphere::VisualSphere;
use crate::data::visual::geometry::visual_clusters::VisualClusters;
use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;
use crate::data::visual::geometry::visual_geometry::VisualGeometry;
use crate::data::visual::geometry::visual_skin::VisualSkin;
use crate::data::visual::geometry::visual_submesh::VisualSubmesh;
use crate::data::visual::geometry::visual_submesh_content::VisualSubmeshContent;
use crate::data::visual::visual_description::VisualDescription;
use crate::pack::buffer::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::buffer::visual_cluster_table::VisualClusterTable;
use crate::pack::buffer::visual_index_window::VisualIndexWindow;
use crate::pack::visual::flat_skin::FlatSkin;
use crate::pack::visual::visual_mesh::VisualMesh;
use crate::pack::visual::visual_package::VisualPackage;
use crate::pack::visual::visual_skeleton::VisualSkeleton;
use crate::pack::visual::visual_skip::VisualSkip;
use crate::pack::visual::visual_tangent_basis::VisualTangentBasis;
use crate::pack::visual_conversion::{convert_declared_bounds, convert_uvs, convert_vector, reverse_triangle_winding};

/// Flattens a parsed OGF visual into renderer ready buffers.
pub struct VisualPacker {}

impl VisualPacker {
  /// Converts an OGF visual into a description and one interleaved byte buffer.
  pub fn pack(file: &OgfFile) -> VisualPackage {
    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();

    // Skinning links name bones of the whole visual, so validating one needs the model's bone count rather than
    // anything the child carries.
    let bone_count: usize = file.bones.as_ref().map_or(0, |it| it.bones.len());

    let submeshes: Vec<VisualSubmesh> = Self::submesh_sources(file)
      .into_iter()
      .enumerate()
      .map(|(index, source)| Self::pack_submesh(&mut builder, index as u32, source, bone_count))
      .collect();

    let computed_bounds: Option<VisualBounds> = Self::compute_bounds(&submeshes);

    let description: VisualDescription = VisualDescription {
      version: file.header.version,
      model_type: file.header.model_type,
      model_type_label: OgfModelType::label(file.header.model_type),
      shader_id: file.header.shader_id,
      source_file: file
        .get_description()
        .map(|it| it.source_file.clone())
        .filter(|it| !it.is_empty()),
      declared_bounds: convert_declared_bounds(&file.header.bounding_box, &file.header.bounding_sphere),
      computed_bounds,
      submeshes,
      bones: file
        .bones
        .as_ref()
        .map(|it| VisualSkeleton::convert_bones(&it.bones, file.ik_data.as_ref().map(|ik| ik.bones.as_slice())))
        .unwrap_or_default(),
      motion_refs: file
        .kinematics
        .as_ref()
        .map(|it| it.motion_refs.clone())
        .unwrap_or_default(),
      // Names come from the motion definitions; a payload's own label is not what the engine resolves.
      embedded_motions: file.get_motion_names().into_iter().map(String::from).collect(),
      buffer_length: builder.length(),
    };

    VisualPackage {
      description,
      buffer: builder.into_buffer(),
    }
  }

  /// The model type a built mesh is packed as: `MT_NORMAL`, drawn whole as it is stored.
  const BUILT_MODEL_TYPE: u8 = 0;

  /// The OGF version a built visual describes itself as, the one every visual the engine loads carries.
  const BUILT_VERSION: u8 = 4;

  /// Packs meshes built rather than read, a submesh each, in the order given, as an OGF visual packs.
  pub fn pack_meshes(meshes: &[VisualMesh]) -> VisualPackage {
    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
    let submeshes: Vec<VisualSubmesh> = meshes
      .iter()
      .enumerate()
      .map(|(index, mesh)| VisualSubmesh {
        index: index as u32,
        model_type: Self::BUILT_MODEL_TYPE,
        model_type_label: OgfModelType::label(Self::BUILT_MODEL_TYPE),
        texture_name: mesh.texture_name.clone(),
        shader_name: mesh.shader_name.clone(),
        content: Self::to_content(Self::pack_mesh(&mut builder, mesh)),
      })
      .collect();
    let computed_bounds: Option<VisualBounds> = Self::compute_bounds(&submeshes);

    VisualPackage {
      description: VisualDescription {
        version: Self::BUILT_VERSION,
        model_type: Self::BUILT_MODEL_TYPE,
        model_type_label: OgfModelType::label(Self::BUILT_MODEL_TYPE),
        shader_id: 0,
        source_file: None,
        // A built mesh declares nothing of itself: what it spans is all there is to declare.
        declared_bounds: computed_bounds.clone().unwrap_or(VisualBounds {
          bounding_box: VisualBox {
            min: Vector3d::ZERO,
            max: Vector3d::ZERO,
          },
          bounding_sphere: VisualSphere {
            center: Vector3d::ZERO,
            radius: 0.0,
          },
        }),
        computed_bounds,
        submeshes,
        bones: Vec::new(),
        motion_refs: Vec::new(),
        embedded_motions: Vec::new(),
        buffer_length: builder.length(),
      },
      buffer: builder.into_buffer(),
    }
  }

  /// One built mesh's attributes, its tangent basis derived from its uvs, and its clusters.
  fn pack_mesh(builder: &mut VisualBufferBuilder, mesh: &VisualMesh) -> Result<VisualGeometry, VisualSkip> {
    let count: usize = mesh.positions.len();

    if count == 0 || mesh.indices.is_empty() || !mesh.indices.len().is_multiple_of(3) {
      return Err(VisualSkip::malformed("A built mesh has no whole triangles"));
    }

    if mesh.normals.len() != count || mesh.uvs.len() != count || mesh.indices.iter().any(|it| usize::from(*it) >= count)
    {
      return Err(VisualSkip::malformed(
        "A built mesh's attributes disagree on its vertices",
      ));
    }

    let positions: Vec<Vector3d> = mesh
      .positions
      .iter()
      .map(|[x, y, z]| Vector3d { x: *x, y: *y, z: *z })
      .collect();
    let bounds: VisualBounds = VisualBounds::from_indexed_positions(&positions, &mesh.indices)
      .ok_or_else(|| VisualSkip::malformed("A built mesh reaches no vertex"))?;
    let (tangents, binormals) = VisualTangentBasis::of(mesh);
    let level: VisualDrawRange = VisualDrawRange {
      start: 0,
      count: mesh.indices.len() as u32,
    };
    let clusters: VisualClusters =
      VisualClusterTable::of_levels(&mesh.indices, &mesh.positions, &[level]).write_into(builder);

    Ok(VisualGeometry {
      vertex_count: count as u32,
      index_count: mesh.indices.len() as u32,
      positions: builder.push_f32_section(mesh.positions.as_flattened()),
      normals: builder.push_f32_section(mesh.normals.as_flattened()),
      tangents: builder.push_f32_section(tangents.as_flattened()),
      binormals: builder.push_f32_section(binormals.as_flattened()),
      uvs: builder.push_f32_section(mesh.uvs.as_flattened()),
      indices: builder.push_u16_section(&mesh.indices),
      skin: None,
      detail_levels: vec![level],
      clusters: Some(clusters),
      bounds,
    })
  }

  /// Drawable pieces of a visual, in the order the file stores them.
  fn submesh_sources(file: &OgfFile) -> Vec<&OgfFile> {
    match file.children.as_ref().map(|it| it.nested.as_slice()) {
      Some(nested) if !nested.is_empty() => nested.iter().collect(),
      _ => vec![file],
    }
  }

  fn pack_submesh(builder: &mut VisualBufferBuilder, index: u32, source: &OgfFile, bone_count: usize) -> VisualSubmesh {
    let model_type: u8 = source.header.model_type;

    VisualSubmesh {
      index,
      model_type,
      model_type_label: OgfModelType::label(model_type),
      texture_name: source.texture.as_ref().map(|it| it.texture_name.clone()),
      shader_name: source.texture.as_ref().map(|it| it.shader_name.clone()),
      content: Self::to_content(Self::pack_geometry(builder, source, bone_count)),
    }
  }

  /// What a submesh holds once packed: its geometry, or why it has none, verbatim.
  fn to_content(packed: Result<VisualGeometry, VisualSkip>) -> VisualSubmeshContent {
    match packed {
      Ok(geometry) => VisualSubmeshContent::Packed { geometry },
      Err(skip) => VisualSubmeshContent::Skipped {
        cause: skip.cause,
        reason: skip.reason,
      },
    }
  }

  /// The extent every packed submesh spans together, or `None` when none produced geometry.
  fn compute_bounds(submeshes: &[VisualSubmesh]) -> Option<VisualBounds> {
    submeshes
      .iter()
      .filter_map(|submesh| submesh.geometry())
      .map(|geometry| geometry.bounds.clone())
      .reduce(VisualBounds::merge)
  }

  /// Convert and append one submesh's attributes, or say why it has none.
  fn pack_geometry(
    builder: &mut VisualBufferBuilder,
    source: &OgfFile,
    bone_count: usize,
  ) -> Result<VisualGeometry, VisualSkip> {
    // Geometry can live in a shared vertex or index container outside the file, in which case the
    // chunk is legitimately absent rather than missing, so none of these are malformed files.
    let geometry: &OgfGeometry = source
      .geometry
      .as_ref()
      .ok_or_else(|| VisualSkip::unsupported("Carries no geometry chunk"))?;

    let vertices: &[OgfVertex] = match (&geometry.vertices, geometry.vertex_format) {
      (Some(OgfVertexData::Skinned(vertices)), _) => vertices,
      (Some(OgfVertexData::Static(_)), Some(format)) => {
        return Err(VisualSkip::unsupported(format!(
          "Static vertex format {format:#010x} is not packed, only skinned meshes are"
        )));
      }
      (_, Some(format)) => {
        return Err(VisualSkip::unsupported(format!(
          "Vertex format {format:#010x} has no known layout"
        )));
      }
      (_, None) => return Err(VisualSkip::unsupported("Carries no vertex chunk")),
    };

    if vertices.is_empty() {
      return Err(VisualSkip::malformed("Vertex chunk is empty"));
    }

    let indices: &Vec<u16> = geometry
      .indices
      .as_ref()
      .ok_or_else(|| VisualSkip::unsupported("Carries no index chunk"))?;

    if indices.is_empty() {
      return Err(VisualSkip::malformed("Index chunk is empty"));
    }

    if !indices.len().is_multiple_of(3) {
      return Err(VisualSkip::malformed(format!(
        "Index count {} is not a whole number of triangles",
        indices.len()
      )));
    }

    let detail_levels: Vec<VisualDrawRange> =
      Self::resolve_detail_levels(source, indices, vertices.len(), source.header.model_type)?;
    let draw_range: VisualDrawRange = detail_levels[0];

    let positions: Vec<Vector3d> = vertices.iter().map(|it| convert_vector(&it.position)).collect();
    let drawn_start: usize = draw_range.start as usize;
    let bounds: VisualBounds = VisualBounds::from_indexed_positions(
      &positions,
      &indices[drawn_start..drawn_start + draw_range.count as usize],
    )
    .ok_or_else(|| VisualSkip::malformed("Drawn range reaches no vertex"))?;

    let flat_positions: Vec<f32> = positions.iter().flat_map(|it| [it.x, it.y, it.z]).collect();
    let flat_normals: Vec<f32> = Self::flatten_directions(vertices, |it| &it.normal);
    let flat_tangents: Vec<f32> = Self::flatten_directions(vertices, |it| &it.tangent);
    let flat_binormals: Vec<f32> = Self::flatten_directions(vertices, |it| &it.binormal);
    let flat_uvs: Vec<f32> = vertices
      .iter()
      .flat_map(|it| {
        let (u, v) = convert_uvs(it.texture_u, it.texture_v);

        [u, v]
      })
      .collect();

    let skin: Option<FlatSkin> = Self::flatten_skin(vertices, bone_count)?;

    let mut wound_indices: Vec<u16> = indices.clone();

    reverse_triangle_winding(&mut wound_indices);

    // Skinned geometry moves with its bones, so only geometry drawn as stored is cut into clusters.
    let clusters: Option<VisualClusters> = skin.is_none().then(|| {
      VisualClusterTable::of_levels(&wound_indices, flat_positions.as_chunks::<3>().0, &detail_levels)
        .write_into(builder)
    });

    Ok(VisualGeometry {
      vertex_count: vertices.len() as u32,
      index_count: wound_indices.len() as u32,
      positions: builder.push_f32_section(&flat_positions),
      normals: builder.push_f32_section(&flat_normals),
      tangents: builder.push_f32_section(&flat_tangents),
      binormals: builder.push_f32_section(&flat_binormals),
      uvs: builder.push_f32_section(&flat_uvs),
      indices: builder.push_u16_section(&wound_indices),
      skin: skin.map(|it| VisualSkin {
        indices: builder.push_u16_section(&it.indices),
        weights: builder.push_f32_section(&it.weights),
      }),
      detail_levels,
      clusters,
      bounds,
    })
  }

  /// One direction of every vertex, converted into renderer space and laid out flat.
  fn flatten_directions(vertices: &[OgfVertex], direction: impl Fn(&OgfVertex) -> &Vector3d) -> Vec<f32> {
    vertices
      .iter()
      .flat_map(|it| {
        let converted: Vector3d = convert_vector(direction(it));

        [converted.x, converted.y, converted.z]
      })
      .collect()
  }

  /// Every vertex's skinning links, widened to four, or `None` when the geometry carries none.
  ///
  /// # Errors
  ///
  /// A link naming a bone the visual does not have is malformed: the engine would index its bone array out of bounds,
  /// and `MeshAssetsVerifier` already reports the same thing as a finding.
  fn flatten_skin(vertices: &[OgfVertex], bone_count: usize) -> Result<Option<FlatSkin>, VisualSkip> {
    if bone_count == 0 || vertices.iter().all(|it| it.links.is_empty()) {
      return Ok(None);
    }

    let mut indices: Vec<u16> = Vec::with_capacity(vertices.len() * VisualSkin::LINKS);
    let mut weights: Vec<f32> = Vec::with_capacity(vertices.len() * VisualSkin::LINKS);

    for vertex in vertices {
      if vertex.links.len() > VisualSkin::LINKS {
        return Err(VisualSkip::malformed(format!(
          "A vertex carries {} skinning links, and the format stores at most {}",
          vertex.links.len(),
          VisualSkin::LINKS
        )));
      }

      for link in &vertex.links {
        if link.bone as usize >= bone_count {
          return Err(VisualSkip::malformed(format!(
            "A vertex is skinned to bone {}, and the skeleton has {bone_count}",
            link.bone
          )));
        }

        indices.push(link.bone);
        weights.push(link.weight);
      }

      for _ in vertex.links.len()..VisualSkin::LINKS {
        indices.push(0);
        weights.push(0.0);
      }
    }

    Ok(Some(FlatSkin { indices, weights }))
  }

  /// The index ranges a submesh may be drawn at, finest first: the whole buffer, or a progressive mesh's usable windows.
  fn resolve_detail_levels(
    source: &OgfFile,
    indices: &[u16],
    vertex_count: usize,
    model_type: u8,
  ) -> Result<Vec<VisualDrawRange>, VisualSkip> {
    let is_progressive: bool = OgfModelType::from_raw(model_type).is_some_and(OgfModelType::is_progressive);

    if !is_progressive {
      let whole: VisualDrawRange = VisualDrawRange {
        start: 0,
        count: indices.len() as u32,
      };

      Self::check_level(indices, whole, vertex_count)
        .map_err(|reason| VisualSkip::malformed(format!("Drawn range {reason}")))?;

      return Ok(vec![whole]);
    }

    // Falling back to the whole buffer would draw every detail level at once, which reads as a fatter
    // model rather than as an error, so a progressive submesh without its table is refused instead.
    let windows: &[OgfSlideWindow] = source
      .swi_data
      .as_ref()
      .map(|swi| swi.windows.as_slice())
      .filter(|windows| !windows.is_empty())
      .ok_or_else(|| {
        VisualSkip::malformed("Progressive geometry carries no detail table, so its full detail range is unknown")
      })?;

    let mut levels: Vec<VisualDrawRange> = Vec::with_capacity(windows.len());

    for (level, window) in windows.iter().enumerate() {
      let range: VisualDrawRange = VisualDrawRange {
        start: window.offset,
        count: u32::from(window.num_tris) * 3,
      };

      match Self::check_level(indices, range, vertex_count) {
        Ok(()) => levels.push(range),
        // The finest level is what the model is: without it there is nothing honest to draw. A coarser one is an
        // option the viewer offers, so a bad one costs the option rather than the submesh.
        Err(reason) => {
          if level == 0 {
            return Err(VisualSkip::malformed(format!("Detail level 0 {reason}")));
          }

          log::warn!("Dropping unusable detail level {level}, which {reason}");
        }
      }
    }

    Ok(levels)
  }

  /// Refuses a range that leaves the index buffer or addresses a vertex the submesh does not have, saying why.
  fn check_level(indices: &[u16], range: VisualDrawRange, vertex_count: usize) -> Result<(), String> {
    VisualIndexWindow {
      count: range.count,
      offset: range.start,
    }
    .select(indices, u32::try_from(vertex_count).unwrap_or(u32::MAX))
    .map(|_| ())
    .map_err(|fault| fault.to_string())
  }
}
