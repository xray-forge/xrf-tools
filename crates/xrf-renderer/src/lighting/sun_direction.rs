use glam::Vec3;

/// The direction sunlight travels, in renderer space, from a weather keyframe's two angles in degrees.
///
/// The engine calls `sun_dir.setHP(sun_altitude, sun_longitude)`, so `sun_altitude` is the heading and `sun_longitude`
/// the pitch, the other way round from what the names say.
pub fn to_renderer_sun_direction(heading: f32, pitch: f32) -> Vec3 {
  let heading: f32 = heading.to_radians();
  let pitch: f32 = pitch.to_radians();

  // `Fvector::setHP`, with engine `z` negated into renderer space.
  Vec3::new(
    -pitch.cos() * heading.sin(),
    pitch.sin(),
    -(pitch.cos() * heading.cos()),
  )
}
