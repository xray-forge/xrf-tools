use serde::Serialize;
use xrf_omf::OmfFile;

/// One motion, joining what the parameters declare to what the motion data holds.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfMotionReport {
  accrue: f32,
  falloff: f32,
  flags: u32,
  keyframes: u32,
  label_diverges: bool,
  name: String,
  power: f32,
  speed: f32,
}

impl OmfMotionReport {
  /// Every motion the container holds, in its order.
  pub fn list(file: &OmfFile) -> Vec<Self> {
    file
      .get_motions()
      .map(|(definition, motion)| Self {
        accrue: definition.accrue,
        falloff: definition.falloff,
        flags: definition.flags,
        keyframes: motion.count,
        label_diverges: !motion.has_label_matching(&definition.name),
        name: definition.name.clone(),
        power: definition.power,
        speed: definition.speed,
      })
      .collect()
  }
}
