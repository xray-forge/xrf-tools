use crate::feature::effects::weather_model_buffers::WeatherModelBuffers;

/// Every bolt model of a weather, by its index among the weather's, and an empty model, which the glows' quads and a
/// bolt without a model bind.
pub(crate) struct ThunderModels {
  pub bolts: Vec<WeatherModelBuffers>,
  pub empty: WeatherModelBuffers,
}

impl ThunderModels {
  /// The model bolt `index` draws, the empty one for none.
  pub(crate) fn get(&self, index: Option<usize>) -> &WeatherModelBuffers {
    index.and_then(|index| self.bolts.get(index)).unwrap_or(&self.empty)
  }
}
