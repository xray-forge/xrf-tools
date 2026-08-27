use serde::Serialize;
use xrf_db::OmfFile;

/// One motion, joining what the parameters declare to what the motion data holds.
///
/// A definition names a motion and how to play it; the keyframe count lives in the payload at the same ordinal, and
/// together they give the effective duration. `labelDiverges` reports that the payload carries a label which is not
/// this motion's name - a stale editing artifact the engine ignores outside `_DEBUG`, and the reason the label is
/// never reported as an identity.
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

/// One skeleton part and the bones it drives.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfPartReport {
  bones: Vec<String>,
  name: String,
}

/// What `omf info` read out of a motion container.
///
/// Motions and parts are listed in full whatever the verbosity: a machine consumer has no
/// `--verbose` to raise, so the per-motion detail a human asks for belongs here unconditionally.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfInfoReport {
  bones: usize,
  motions: Vec<OmfMotionReport>,
  parts: Vec<OmfPartReport>,
  version: u16,
}

impl OmfInfoReport {
  pub fn new(file: &OmfFile) -> Self {
    Self {
      bones: file.parameters.get_bones_count(),
      motions: file
        .get_motions()
        .map(|(definition, motion)| OmfMotionReport {
          accrue: definition.accrue,
          falloff: definition.falloff,
          flags: definition.flags,
          keyframes: motion.count,
          label_diverges: !motion.has_label_matching(&definition.name),
          name: definition.name.clone(),
          power: definition.power,
          speed: definition.speed,
        })
        .collect(),
      parts: file
        .parameters
        .parts
        .iter()
        .map(|part| OmfPartReport {
          bones: part.get_bones().into_iter().map(String::from).collect(),
          name: part.name.clone(),
        })
        .collect(),
      version: file.parameters.version,
    }
  }
}
