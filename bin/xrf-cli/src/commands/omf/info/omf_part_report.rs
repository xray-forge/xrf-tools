use serde::Serialize;
use xrf_omf::OmfFile;

/// One skeleton part and the bones it drives.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfPartReport {
  bones: Vec<String>,
  name: String,
}

impl OmfPartReport {
  /// Every part the container's parameters declare, in their order.
  pub fn list(file: &OmfFile) -> Vec<Self> {
    file
      .parameters
      .parts
      .iter()
      .map(|part| Self {
        bones: part.get_bones().into_iter().map(String::from).collect(),
        name: part.name.clone(),
      })
      .collect()
  }
}
