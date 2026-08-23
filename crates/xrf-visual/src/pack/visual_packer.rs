use xrf_db::{OgfFile, OgfGeometry, OgfSlideWindow, OgfVertex, Vector3d};

use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_description::VisualDescription;
use crate::data::visual_model_type::VisualModelType;
use crate::data::visual_section::VisualDrawRange;
use crate::data::visual_submesh::{VisualGeometry, VisualSkin, VisualSkipCause, VisualSubmesh, VisualSubmeshContent};
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_declared_bounds, convert_texture_coordinates, convert_vector};
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
///
/// Packing never fails. A child that carries nothing drawable becomes a submesh holding the reason it
/// was skipped, so one broken piece neither hides the rest of a model nor turns into an error the
/// caller has to interpret.
pub struct VisualPacker {}

impl VisualPacker {
  /// Skinning links per vertex, which is both the format ceiling (`vertBoned4W`) and the width of a renderer's
  /// `vec4` skin attributes.
  const SKIN_LINKS: usize = 4;

  /// Converts an OGF visual into a description and one interleaved byte buffer.
  ///
  /// Submeshes retain source order. Unsupported or malformed geometry is represented in the
  /// description as `Skipped`, while drawable submeshes reference aligned ranges in the returned buffer.
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
      model_type_label: VisualModelType::label(file.header.model_type),
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
      embedded_motions: file
        .motions
        .as_ref()
        .map(|it| it.motions.iter().map(|motion| motion.name.clone()).collect())
        .unwrap_or_default(),
      buffer_length: builder.length(),
    };

    VisualPackage {
      description,
      buffer: builder.into_buffer(),
    }
  }

  /// Drawable pieces of a visual, in the order the file stores them.
  ///
  /// A skeleton keeps its geometry on children and carries none itself; a single level visual is its
  /// own only piece. Submesh order is the child order, because a texture or shader reference is only
  /// meaningful against the child it came from.
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
      model_type_label: VisualModelType::label(model_type),
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
  ///
  /// The error type is the reason a consumer displays, not a failure: every early return here ends up
  /// beside the submesh in the description.
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
    let flat_normals: Vec<f32> = vertices
      .iter()
      .flat_map(|it| {
        let normal: Vector3d = convert_vector(&it.normal);

        [normal.x, normal.y, normal.z]
      })
      .collect();
    let flat_uvs: Vec<f32> = vertices
      .iter()
      .flat_map(|it| {
        let (u, v) = convert_texture_coordinates(it.texture_u, it.texture_v);

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

  /// Every vertex's skinning links, widened to four, or `None` when the geometry carries none.
  ///
  /// Widened rather than stored at their natural width because a renderer's skin attributes are `vec4` whatever the
  /// source layout was, and a padding link at weight zero moves nothing. Whether a submesh is skinned at all is read
  /// off the vertices rather than off the vertex format, so a layout added to the reader later needs no change here.
  ///
  /// A visual with no bone list gets no skin either: a link is an index into that list, so without one it names
  /// nothing, and geometry that cannot be posed is better drawn as it is stored than bound to a skeleton of no bones.
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
  ///
  /// Static geometry draws its whole buffer. Progressive geometry stores every detail level in that
  /// same buffer with the finest one at level zero, which `FSkinned.cpp:419` selects when it wants
  /// full geometry, so drawing all of it would stack the coarse shells over the fine mesh.
  fn resolve_detail_levels(
    source: &OgfFile,
    indices: &[u16],
    vertex_count: usize,
    model_type: u8,
  ) -> Result<Vec<VisualDrawRange>, VisualSkip> {
    let is_progressive: bool = VisualModelType::from_raw(model_type).is_some_and(VisualModelType::is_progressive);

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
  ///
  /// Returns the reason rather than a skip, because whether an unusable level fails its submesh or merely
  /// disappears from the choices depends on which level it is, which the caller knows and this does not.
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

/// Reverse the winding of every triangle in place.
///
/// Mirroring Z to reach three.js space flips the orientation of every triangle, and swapping the
/// second and third index of each triple restores it. Reversing the array as a whole would give the
/// same per triangle winding while moving every triangle, which silently invalidates every detail
/// table offset into the buffer.
pub(crate) fn reverse_triangle_winding(indices: &mut [u16]) {
  for triangle in indices.chunks_exact_mut(3) {
    triangle.swap(1, 2);
  }
}
