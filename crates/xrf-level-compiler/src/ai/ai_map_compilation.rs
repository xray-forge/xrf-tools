use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_guid::AiMapGuid;

/// A compiled `level.ai`, where its identity came from, and the GPU that traced its covers, if one did.
#[derive(Clone, Debug, PartialEq)]
pub struct AiMapCompilation {
  pub map: AiMap,
  pub guid: AiMapGuid,
  pub gpu: Option<String>,
}
