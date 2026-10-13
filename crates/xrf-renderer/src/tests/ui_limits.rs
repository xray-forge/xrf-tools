use std::fs;
use std::path::PathBuf;

use crate::contract::render_ambient_occlusion_vbao_settings::RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION;
use crate::contract::render_contact_shadow_settings::RENDER_MAX_CONTACT_SHADOW_LIGHTS;
use crate::contract::render_shadow_settings::{RENDER_MAX_SHADOW_CASCADES, RENDER_MAX_SHADOW_FILTER};
use crate::frame::periodic_scheduler::PeriodicScheduler;

/// Where the UI imports the renderer's limits from, beside its other render settings.
const UI_LIMITS: &str = "../../bin/xrf-ui/src/core/render/lib/settings/render-contract-limits.ts";

/// The renderer's limits the UI bounds its settings by, as the TypeScript module the UI imports them from.
fn write_ui_limits() -> String {
  let limits: [(&str, &str, String); 5] = [
    (
      "Cascades the sun's shadow can be cut into at most: the sun reads their texels from one `vec4`.",
      "RENDER_MAX_SHADOW_CASCADES",
      RENDER_MAX_SHADOW_CASCADES.to_string(),
    ),
    (
      "Texels the sun's shadow filter reaches each way at most: the widest the sun's pass has a pipeline for.",
      "RENDER_MAX_SHADOW_FILTER",
      RENDER_MAX_SHADOW_FILTER.to_string(),
    ),
    (
      "Local lights a pixel marches contact shadows towards at most: the lights' shader keeps their weights in an array.",
      "RENDER_MAX_CONTACT_SHADOW_LIGHTS",
      RENDER_MAX_CONTACT_SHADOW_LIGHTS.to_string(),
    ),
    (
      "Frames VBAO accumulates over at most.",
      "RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION",
      RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION.to_string(),
    ),
    (
      "GPU milliseconds a frame gives the work drawn once in so many frames that may wait: the renderer's default.",
      "DEFAULT_PERIODIC_BUDGET",
      PeriodicScheduler::DEFAULT_BUDGET.to_string(),
    ),
  ];
  let mut module: String = String::from(
    "// Auto-generated from the renderer's contract by `cargo test -p xrf-renderer`. Do not edit it manually.\n",
  );

  for (description, name, value) in limits {
    module.push_str(&format!(
      "\n/** {description} */\nexport const {name}: number = {value};\n"
    ));
  }

  module
}

/// The UI's copy of the renderer's limits is what the contract writes: a stale one is rewritten and fails this once.
#[test]
fn keeps_the_ui_limits_up_to_date() {
  let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(UI_LIMITS);
  let expected: String = write_ui_limits();
  let current: String = fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");

  if current != expected {
    fs::write(&path, &expected).expect("the UI's limits module is writable");

    panic!("{UI_LIMITS} was stale and is rewritten: commit it");
  }
}
