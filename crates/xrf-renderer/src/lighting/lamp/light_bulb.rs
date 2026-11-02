use xrf_visual::LightDescription;

/// Metres about a lamp's position its light leaves from: a bulb's.
const LAMP_RADIUS: f32 = 0.05;
/// A campfire's: its bed of embers.
const CAMPFIRE_RADIUS: f32 = 0.25;
/// A zone's idle light's: the glow of the anomaly it stands in.
const ZONE_RADIUS: f32 = 0.5;

/// Metres about a light's position its light leaves from, by what gives it: a campfire's bed, an anomaly's glow or a
/// lamp's bulb. The physical light's falloff stops growing within it, and a traced shadow's rays are drawn over it, so
/// a shadow's penumbra widens with its caster's distance from the light.
pub fn get_bulb_radius(light: &LightDescription) -> f32 {
  if light.campfire.is_some() {
    CAMPFIRE_RADIUS
  } else if light.zone_sphere.is_some() || light.motion.is_some() {
    ZONE_RADIUS
  } else {
    LAMP_RADIUS
  }
}
