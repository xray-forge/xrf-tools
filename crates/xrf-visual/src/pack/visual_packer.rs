use xrf_math::Vector3d;
use xrf_ogf::{OgfFile, OgfGeometry, OgfModelType, OgfSlideWindow, OgfVertex};

use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_description::VisualDescription;
use crate::data::visual_section::VisualDrawRange;
use crate::data::visual_submesh::{VisualGeometry, VisualSkin, VisualSkipCause, VisualSubmesh, VisualSubmeshContent};
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{
  convert_declared_bounds, convert_uvs, convert_vector, reverse_triangle_winding,
};
use crate::pack::visual_package::VisualPackage;
use crate::pack::visual_skeleton::convert_bones;

/// A submesh that produced no geometry, as the packer's internal early return.
///
/// Becomes [`VisualSubmeshContent::Skipped`] verbatim, so every reason below is one a consumer reads.
struct VisualSkip {
  cause: VisualSkipCause,
  reason: String,
}

impl VisualSkip {
  /// Geometry the packer cannot read, which is a gap in coverage rather than a broken file.
  fn unsupported(reason: impl Into<String>) -> Self {
    Self {
      cause: VisualSkipCause::Unsupported,
      reason: reason.into(),
    }
  }

  /// Geometry that contradicts itself, which no amount of added coverage would fix.
  fn malformed(reason: impl Into<String>) -> Self {
    Self {
      cause: VisualSkipCause::Malformed,
      reason: reason.into(),
    }
  }
}

/// One submesh's skinning links, flattened four per vertex and paired so neither can be pushed without the other.
struct FlatSkin {
  indices: Vec<u16>,
  weights: Vec<f32>,
}

/// Flattens a parsed OGF visual into renderer ready buffers.
pub struct VisualPacker {}

impl VisualPacker {
  /// Skinning links per vertex, which is both the format ceiling (`vertBoned4W`) and the width of a renderer's
  /// `vec4` skin attributes.
  const SKIN_LINKS: usize = 4;

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

    let computed_bounds: Option<VisualBounds> = submeshes
      .iter()
      .filter_map(|submesh| submesh.geometry())
      .map(|geometry| geometry.bounds.clone())
      .reduce(VisualBounds::merge);

    let description: VisualDescription = VisualDescription {
      version: file.header.version,
      model_type: file.header.model_type,
      model_type_label: OgfModelType::label(file.header.model_type),
      shader_id: file.header.shader_id,
      source_file: file
        .description
        .as_ref()
        .map(|it| it.source_file.clone())
        .filter(|it| !it.is_empty()),
      declared_bounds: convert_declared_bounds(&file.header.bounding_box, &file.header.bounding_sphere),
      computed_bounds,
      submeshes,
      bones: file
        .bones
        .as_ref()
        .map(|it| convert_bones(&it.bones, file.ik_data.as_ref().map(|ik| ik.bones.as_slice())))
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
      content: match Self::pack_geometry(builder, source, bone_count) {
        Ok(geometry) => VisualSubmeshContent::Packed { geometry },
        Err(skip) => VisualSubmeshContent::Skipped {
          cause: skip.cause,
          reason: skip.reason,
        },
      },
    }
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

    let vertices: &Vec<OgfVertex> = geometry.vertices.as_ref().ok_or_else(|| match geometry.vertex_format {
      Some(format) => VisualSkip::unsupported(format!("Vertex format {format:#010x} has no known layout")),
      None => VisualSkip::unsupported("Carries no vertex chunk"),
    })?;

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

    let mut indices: Vec<u16> = Vec::with_capacity(vertices.len() * Self::SKIN_LINKS);
    let mut weights: Vec<f32> = Vec::with_capacity(vertices.len() * Self::SKIN_LINKS);

    for vertex in vertices {
      if vertex.links.len() > Self::SKIN_LINKS {
        return Err(VisualSkip::malformed(format!(
          "A vertex carries {} skinning links, and the format stores at most {}",
          vertex.links.len(),
          Self::SKIN_LINKS
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

      for _ in vertex.links.len()..Self::SKIN_LINKS {
        indices.push(0);
        weights.push(0.0);
      }
    }

    Ok(Some(FlatSkin { indices, weights }))
  }

  /// The index range that draws a submesh at full detail.
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

      Self::assert_level_in_range(indices, whole, vertex_count)
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

      match Self::assert_level_in_range(indices, range, vertex_count) {
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

  /// Reject a range that leaves the index buffer or addresses a vertex the submesh does not have.
  fn assert_level_in_range(indices: &[u16], range: VisualDrawRange, vertex_count: usize) -> Result<(), String> {
    let start: usize = range.start as usize;
    let count: usize = range.count as usize;

    if range.start as u64 + range.count as u64 > indices.len() as u64 {
      return Err(format!(
        "draws {count} indices from offset {start}, past the {} the index chunk holds",
        indices.len()
      ));
    }

    match indices[start..start + count]
      .iter()
      .copied()
      .find(|index| *index as usize >= vertex_count)
    {
      Some(index) => Err(format!(
        "references vertex {index}, past the {vertex_count} the vertex chunk holds"
      )),
      None => Ok(()),
    }
  }
}
