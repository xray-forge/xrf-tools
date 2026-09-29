use serde::Serialize;
use xrf_ltx::Condlist;

use crate::level::level_weather_option::LevelWeatherOption;
use crate::level::weather_graph::WeatherGraph;
use crate::level::weather_graphs::WeatherGraphs;

/// The cycles a level can be lit under, as its game's weather script chooses between them: the `weathers` key of its
/// `game.ltx` section, a condlist, read against `dynamic_weather_graphs.ltx`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWeather {
  pub level: String,
  /// The `weathers` key as written, `[default]` where the level writes none.
  pub key: String,
  /// Every cycle the key can play, in the order its branches and graphs list them; the first is where a viewer starts.
  pub options: Vec<LevelWeatherOption>,
}

impl LevelWeather {
  /// What `level_weathers.script` reads for a level that writes no `weathers`.
  pub const DEFAULT_KEY: &'static str = "[default]";

  /// Resolves a level's `weathers`: each result its condlist can pick, a graph to its states, `atmosfear` on Monolith
  /// to its presets, anything else to the cycle it names. A condlist that does not parse is taken as one name.
  pub fn resolve(level: &str, key: &str, graphs: &WeatherGraphs) -> Self {
    let key: String = key.trim().to_owned();
    let picks: Vec<String> = match Condlist::parse(&key) {
      Ok(condlist) => condlist
        .branches
        .into_iter()
        .filter_map(|branch| branch.result)
        .collect(),
      Err(_) => vec![key.clone()],
    };

    let mut options: Vec<LevelWeatherOption> = Vec::new();

    for pick in picks {
      for option in Self::resolve_pick(&pick, graphs) {
        if !options.contains(&option) {
          options.push(option);
        }
      }
    }

    Self {
      key,
      level: level.to_owned(),
      options,
    }
  }

  /// Atmosfear's states are read only on Monolith, so a vanilla read has none to resolve `atmosfear` to.
  fn resolve_pick(pick: &str, graphs: &WeatherGraphs) -> Vec<LevelWeatherOption> {
    if pick == WeatherGraphs::ATMOSFEAR && !graphs.atmosfear.is_empty() {
      return graphs
        .atmosfear
        .iter()
        .flat_map(|cycle| {
          cycle.presets.iter().map(|preset| LevelWeatherOption {
            cycle: preset.clone(),
            graph: Some(WeatherGraphs::ATMOSFEAR.to_owned()),
            state: Some(cycle.state.clone()),
          })
        })
        .collect();
    }

    if let Some(graph) = graphs.find_graph(pick) {
      return graph
        .states
        .iter()
        .map(|state| LevelWeatherOption {
          cycle: WeatherGraph::cycle_of(&state.state),
          graph: Some(graph.name.clone()),
          state: Some(state.state.clone()),
        })
        .collect();
    }

    vec![LevelWeatherOption {
      cycle: pick.to_owned(),
      graph: None,
      state: None,
    }]
  }
}
