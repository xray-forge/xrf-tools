use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_guid::AiMapGuid;

/// A compiled `level.ai`, and where its identity came from.
#[derive(Clone, Debug, PartialEq)]
pub struct AiMapCompilation {
  pub map: AiMap,
  pub guid: AiMapGuid,
}
