use std::ptr;
use std::sync::{Arc, Weak};

use crate::host::weather::render_level_weather::RenderLevelWeather;

/// What a view builds once from the level's weather, with the weather it was built for, held weakly so its address is
/// never another's.
pub struct WeatherBuilt<T> {
  weather: Weak<RenderLevelWeather>,
  value: T,
}

impl<T> WeatherBuilt<T> {
  /// What `slot` holds for `weather`, built by `build` where it holds nothing or what another weather was built into.
  pub fn follow<'a>(slot: &'a mut Option<Self>, weather: &Arc<RenderLevelWeather>, build: impl FnOnce() -> T) -> &'a T {
    if slot
      .as_ref()
      .is_some_and(|built| !ptr::eq(built.weather.as_ptr(), Arc::as_ptr(weather)))
    {
      *slot = None;
    }

    &slot
      .get_or_insert_with(|| Self {
        weather: Arc::downgrade(weather),
        value: build(),
      })
      .value
  }

  pub fn get(&self) -> &T {
    &self.value
  }
}
