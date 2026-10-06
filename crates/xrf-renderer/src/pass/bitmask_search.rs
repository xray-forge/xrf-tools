use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::bitmask_pass_names::BitmaskPassNames;

/// What a frame's visibility-bitmask search yields: VBAO's occlusion, the indirect light, or both from one search.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitmaskSearch {
  /// Whether its visibility is the frame's ambient occlusion.
  pub is_occluding: bool,
  /// Whether it gathers the indirect light.
  pub is_lit: bool,
}

impl BitmaskSearch {
  /// The pass that copies the frame's light for the search to gather.
  pub const SOURCE_PASS: &'static str = "indirect source";

  /// What the view searches for, or none where it searches nothing: unlit, wireframe, or neither VBAO nor the indirect
  /// light asked for.
  pub fn new(options: &RenderViewOptions) -> Option<Self> {
    if !options.mode.is_lit || options.mode.is_wireframe {
      return None;
    }

    let occlusion = &options.features.ambient_occlusion;
    let search: Self = Self {
      is_occluding: occlusion.is_enabled && occlusion.is_vbao(),
      is_lit: options.features.indirect_light.is_drawn(),
    };

    (search.is_occluding || search.is_lit).then_some(search)
  }

  pub fn get_names(self) -> BitmaskPassNames {
    match (self.is_occluding, self.is_lit) {
      (true, true) => BitmaskPassNames {
        search: "vbao + indirect",
        accumulate: "vbao + indirect accumulate",
        filter: "vbao + indirect filter",
      },
      (false, true) => BitmaskPassNames {
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
