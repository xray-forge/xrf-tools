use serde::Serialize;

use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_model_type::VisualModelType;
use crate::data::visual_section::{VisualDrawRange, VisualSection};

/// Where one submesh's skinning links sit in the geometry buffer.
///
/// Four per vertex whatever the source layout stores, because that is the width a renderer's skin attributes have:
/// a vertex with fewer links is padded with bone zero at weight zero, which contributes nothing. Indices are `u16`
/// into the visual's own bone list - the engine looks a link up as `LL_GetBoneInstance(v.matrix)`
/// (`xray-16/src/Layers/xrRender/SkeletonX.cpp:359`), so they are global to the model rather than local to the
/// submesh - and each vertex's weights sum to one, the last one having been reconstructed by the reader.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSkin {
  pub indices: VisualSection,
  pub weights: VisualSection,
}

/// Where one submesh's attributes sit inside the geometry buffer, and what to draw from them.
///
/// Every section is a byte range into the one buffer the model ships as, so a consumer builds views
/// over it without copying. `indices` covers the whole index buffer, including the coarser detail
/// levels a progressive submesh carries; [`Self::detail_levels`] names which slices of it are
/// drawable, and a consumer that does not want to choose draws [`Self::get_default_level`].
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualGeometry {
  pub vertex_count: u32,
  pub index_count: u32,
  pub positions: VisualSection,
  pub normals: VisualSection,
  pub uvs: VisualSection,
  pub indices: VisualSection,
  /// Skinning links, or `None` for geometry that carries none and is therefore drawn as it is stored.
  pub skin: Option<VisualSkin>,
  /// Every range a consumer may draw, finest first, and never empty.
  ///
  /// A static submesh has exactly one: its whole index buffer. A progressive one has a range per detail level of
  /// its slide-window table, coarsening as the index rises. Each is validated here — inside the index buffer, and
  /// reaching no vertex the submesh lacks — so choosing a level is a choice between drawable ranges rather than a
  /// range check the consumer has to remember. A coarse level that fails validation is left out rather than
  /// failing the submesh, so a model with one bad level still renders at the levels that are sound.
  pub detail_levels: Vec<VisualDrawRange>,
  pub bounds: VisualBounds,
}

impl VisualGeometry {
  /// The range drawn unless a consumer picks another level: the finest one.
  ///
  /// # Panics
  ///
  /// Never in practice. [`Self::detail_levels`] is non-empty by construction - a submesh whose finest level does
  /// not validate is reported as skipped rather than packed - and this states that invariant where it is relied on.
  pub fn get_default_level(&self) -> VisualDrawRange {
    self.detail_levels[0]
  }
}

/// Why a submesh produced no geometry, graded so a caller does not read the message to find out.
///
/// The distinction is what separates a gap in this crate's coverage from a file that contradicts
/// itself, which is the difference between a sweep noting something and a sweep failing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VisualSkipCause {
  /// Geometry is stored in a form the packer does not handle, such as a shared vertex or index
  /// container living outside the file.
  Unsupported,
  /// Geometry contradicts itself, such as a detail level reaching past the index buffer it indexes.
  Malformed,
}

/// Whether a submesh produced drawable geometry, and why not when it did not.
///
/// A child that cannot be packed is a value rather than an error so the rest of a model still
/// renders, and so the reason reaches the panel that lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VisualSubmeshContent {
  Packed { geometry: VisualGeometry },
  Skipped { cause: VisualSkipCause, reason: String },
}

/// One drawable piece of a visual: a child of a skeleton, or a whole single level visual.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSubmesh {
  pub index: u32,
  pub model_type: u8,
  pub model_type_label: String,
  /// X-Ray logical texture path, without an extension. A skeleton keeps these on its children rather
  /// than at the top level, which is why a skeleton's own texture chunk is usually absent.
  pub texture_name: Option<String>,
  pub shader_name: Option<String>,
  pub content: VisualSubmeshContent,
}

impl VisualSubmesh {
  /// Whether the submesh stores its geometry as a progressive mesh, which its model type decides.
  ///
  /// Asked here rather than inferred from the detail table: a static submesh that happens to carry a slide-window
  /// chunk is not progressive, and a progressive one whose coarse levels were all unusable still is.
  pub fn is_progressive(&self) -> bool {
    VisualModelType::from_raw(self.model_type).is_some_and(VisualModelType::is_progressive)
  }

  pub fn geometry(&self) -> Option<&VisualGeometry> {
    match &self.content {
      VisualSubmeshContent::Packed { geometry } => Some(geometry),
      VisualSubmeshContent::Skipped { .. } => None,
    }
  }

  pub fn skipped(&self) -> Option<(VisualSkipCause, &str)> {
    match &self.content {
      VisualSubmeshContent::Packed { .. } => None,
      VisualSubmeshContent::Skipped { cause, reason } => Some((*cause, reason)),
    }
  }

  #[cfg(test)]
  pub(crate) fn skipped_reason(&self) -> Option<&str> {
    self.skipped().map(|(_, reason)| reason)
  }
}
