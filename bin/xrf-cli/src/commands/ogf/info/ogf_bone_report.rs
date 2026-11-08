use serde::Serialize;
use xrf_ogf::OgfBone;

/// One bone and the bone it hangs from.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OgfBoneReport {
  name: String,
  parent: String,
}

impl OgfBoneReport {
  pub(crate) fn new(bone: &OgfBone) -> Self {
    Self {
      name: bone.name.clone(),
      parent: bone.parent.clone(),
    }
  }
}
