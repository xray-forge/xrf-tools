use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_ltx::{Ltx, Section, read_engine_float, scan_engine_float};

use crate::level::atmosfear_cycle::AtmosfearCycle;
use crate::level::weather_graph::WeatherGraph;
use crate::level::weather_graph_state::WeatherGraphState;

/// `environment\dynamic_weather_graphs.ltx`, which the weather scripts choose a level's cycle from; the engine never
/// reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherGraphs {
  /// Every section whose lines are all states and weights.
  pub graphs: Vec<WeatherGraph>,
  /// Atmosfear's states and their cycles, on Anomaly.
  pub atmosfear: Vec<AtmosfearCycle>,
}

impl WeatherGraphs {
  /// The key a level writes to be played by Atmosfear, and the section listing its states.
  pub const ATMOSFEAR: &'static str = "atmosfear";
  const ATMOSFEAR_CYCLES: &'static str = "weather_cycles";

  /// Reads the graphs, and Atmosfear's states where the engine is Monolith's.
  pub(crate) fn read(ltx: &Ltx, engine: XrayEngine) -> Self {
    let graphs: Vec<WeatherGraph> = ltx
      .iter()
      .filter(|(name, section)| !name.is_empty() && !section.is_empty() && Self::is_graph(section))
      .map(|(name, section)| WeatherGraph {
        name: name.to_owned(),
        states: section
          .iter()
          .map(|(state, weight)| WeatherGraphState {
            state: state.to_owned(),
            weight: read_engine_float(weight),
          })
          .collect(),
      })
      .collect();

    let atmosfear: Vec<AtmosfearCycle> = match (engine, ltx.section(Self::ATMOSFEAR_CYCLES)) {
      (XrayEngine::Extended, Some(states)) => states
        .iter()
        .map(|(state, _)| AtmosfearCycle {
          presets: ltx
            .section(&format!("cycle_{state}"))
            .map(|presets| presets.iter().map(|(cycle, _)| cycle.to_owned()).collect())
            .unwrap_or_default(),
          state: state.to_owned(),
        })
        .collect(),
      _ => Vec::new(),
    };

    Self { atmosfear, graphs }
  }

  /// The graph of a name, where there is one.
  pub fn find_graph(&self, name: &str) -> Option<&WeatherGraph> {
    self.graphs.iter().find(|graph| graph.name == name)
  }

  /// Whether a section is a graph: every line a state and a number, as the script reads its weights.
  fn is_graph(section: &Section) -> bool {
    section.iter().all(|(_, weight)| {
      let weight: &str = weight.trim();

      scan_engine_float(weight).is_some_and(|(_, used)| used == weight.len())
    })
  }
}
