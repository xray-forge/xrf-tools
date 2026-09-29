use crate::key::declare_environment_keys;

declare_environment_keys! {
  /// The keys of one hour of Monolith's sun table, `CEnvironment::load_sun` (`xrEngine/Environment_misc.cpp`); OpenXRay
  /// has no table and reads neither.
  pub enum SunPositionKey {
    /// Degrees; the heading, whatever the name says, as `setHP` takes it.
    SunAltitude = "sun_altitude": Number, vanilla: Unread, extended: Required(Number(0.0));
    /// Degrees; the pitch.
    SunLongitude = "sun_longitude": Number, vanilla: Unread, extended: Required(Number(0.0));
  }
}
