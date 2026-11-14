use serde::Serialize;
use xrf_level::LevelAiGraphCensus;
use xrf_report::Report;

use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::ai_map_guid::AiMapGuid;
use crate::detail::detail_slot_counts::DetailSlotCounts;
use crate::geometry::level::geometry_level_census::GeometryLevelCensus;
use crate::light::bake::light_bake_census::LightBakeCensus;

/// What ran a step of a level's compile, and what it says of the run.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelCompileStepRun {
  /// The detail lighter (`DetailLighter`), how it met the slots, and the GPU that traced its shadow rays, none where
  /// the CPU traced them.
  DetailLighter {
    slots: DetailSlotCounts,
    gpu: Option<String>,
  },
  /// The geometry compiler (`GeometryLevel`), and what it made.
  GeometryCompiler { census: GeometryLevelCensus },
  /// The light baker (`LightBaker`), run within the geometry compile, what it made, and the GPU that traced its
  /// shadow rays, none where the CPU traced them.
  LightBaker {
    census: LightBakeCensus,
    gpu: Option<String>,
  },
  /// The AI map compiler (`AiMapCompiler`): the covers it computed, the GPU that traced their rays, none where
  /// the CPU did or no ray was traced, where the map's identity came from, and the graph check run on the map it wrote,
  /// with what the map held.
  AiMapCompiler {
    covers: AiMapCovers,
    gpu: Option<String>,
    guid: AiMapGuid,
    census: LevelAiGraphCensus,
    graph: Report,
  },
}
