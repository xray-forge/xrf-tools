use serde::{Deserialize, Serialize};
use xrf_renderer_core::GraphReport;

use crate::contract::render_graph_pass::RenderGraphPass;
use crate::contract::render_graph_transient::RenderGraphTransient;

/// What the frame graph made of a frame a viewport drew: its passes and its window's, in the order they ran, the
/// passes culled, the encode groups, the render passes, and the transients the whole frame made with the pooled
/// resources they share.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphReport {
  pub passes: Vec<RenderGraphPass>,
  pub culled: Vec<String>,
  pub groups: Vec<String>,
  /// Render passes the whole frame began.
  pub render_pass_count: u32,
  pub transients: Vec<RenderGraphTransient>,
  /// Bytes the transients would take apart, and the pooled textures and buffers they share and their bytes.
  pub transient_bytes: u64,
  pub pooled_count: u32,
  pub pooled_bytes: u64,
}

impl RenderGraphReport {
  /// A frame's report as one viewport reads it: its own passes, `owner`, and those of the frame's own.
  pub fn of(report: &GraphReport, owner: u32, frame_owner: u32) -> Self {
    Self {
      passes: report
        .passes
        .iter()
        .filter(|pass| pass.owner == owner || pass.owner == frame_owner)
        .map(|pass| RenderGraphPass {
          name: pass.name.clone(),
          kind: pass.kind.into(),
          group: pass.group.clone(),
          render_pass: pass.render_pass.map(|it| it as u32),
        })
        .collect(),
      culled: report.culled.clone(),
      groups: report.groups.clone(),
      render_pass_count: report.render_pass_count as u32,
      transients: report
        .transients
        .iter()
        .map(|transient| RenderGraphTransient {
          label: transient.label.clone(),
          is_texture: transient.is_texture,
          ordinal: transient.ordinal as u32,
          first: transient.first as u32,
          last: transient.last as u32,
          bytes: transient.bytes,
        })
        .collect(),
      transient_bytes: report.transient_bytes,
      pooled_count: report.pooled_count as u32,
      pooled_bytes: report.pooled_bytes,
    }
  }
}
