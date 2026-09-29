/// Where a cycle is mixed: at a time of day, seen from a point the level's modifiers reach or not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherMixPoint {
  /// Seconds, wrapped into the day.
  pub time: f32,
  /// In engine space.
  pub view: [f32; 3],
}
