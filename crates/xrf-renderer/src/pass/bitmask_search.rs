use crate::context::gpu_tracing::GpuTracing;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::bitmask_pass_names::BitmaskPassNames;

/// What a frame's visibility-bitmask search yields: VBAO's occlusion, the indirect light, or both from one search; or
/// RTAO's occlusion traced in the search's place, accumulated and filtered as the search's is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitmaskSearch {
  /// Whether its visibility is the frame's ambient occlusion.
  pub is_occluding: bool,
  /// Whether it gathers the indirect light.
  pub is_lit: bool,
  /// Whether its occlusion is traced rather than searched: RTAO asked for on a GPU that traces rays.
  pub is_traced: bool,
}

impl BitmaskSearch {
  /// The pass that copies the frame's light for the search to gather.
  pub const SOURCE_PASS: &'static str = "indirect source";
  /// The pass tracing the occlusion's rays.
  pub const TRACE_PASS: &'static str = "rtao";

  /// What the view searches for on a GPU with the `tracing` given, or none where it searches nothing: unlit,
  /// wireframe, or neither VBAO, RTAO nor the indirect light asked for.
  pub fn new(options: &RenderViewOptions, tracing: GpuTracing) -> Option<Self> {
    if !options.mode.is_lit || options.mode.is_wireframe {
      return None;
    }

    let occlusion = &options.features.ambient_occlusion;
    let is_occluding: bool = occlusion.is_enabled && !occlusion.is_gtao();
    let search: Self = Self {
      is_occluding,
      is_lit: options.features.indirect_light.is_drawn(),
      is_traced: is_occluding && occlusion.is_traced() && tracing.is_available(),
    };

    (search.is_occluding || search.is_lit).then_some(search)
  }

  pub fn get_names(self) -> BitmaskPassNames {
    match (self.is_occluding, self.is_lit, self.is_traced) {
      (true, true, true) => BitmaskPassNames {
        search: "indirect light",
        accumulate: "rtao + indirect accumulate",
        filter: "rtao + indirect filter",
      },
      (true, false, true) => BitmaskPassNames {
        search: "vbao",
        accumulate: "rtao accumulate",
        filter: "rtao filter",
      },
      (true, true, false) => BitmaskPassNames {
        search: "vbao + indirect",
        accumulate: "vbao + indirect accumulate",
        filter: "vbao + indirect filter",
      },
      (false, true, _) => BitmaskPassNames {
        search: "indirect light",
        accumulate: "indirect accumulate",
        filter: "indirect filter",
      },
      _ => BitmaskPassNames {
        search: "vbao",
        accumulate: "vbao accumulate",
        filter: "vbao filter",
      },
    }
  }
}
