#import "common/lighting"

// The sky's irradiance as order-2 spherical harmonics (`frame/sky_irradiance.wgsl` projects it, `SkyHarmonics`): the
// nine real harmonics a direction is weighed by, and the irradiance over π a normal facing them takes.

// The nine real spherical harmonics of order 2 at a unit direction.
fn sky_harmonics_basis(direction: vec3<f32>) -> array<f32, 9> {
  let x: f32 = direction.x;
  let y: f32 = direction.y;
  let z: f32 = direction.z;

  return array<f32, 9>(
    0.282095,
    0.488603 * y,
    0.488603 * z,
    0.488603 * x,
    1.092548 * x * y,
    1.092548 * y * z,
    0.315392 * (3.0 * z * z - 1.0),
    1.092548 * x * z,
    0.546274 * (x * x - y * y),
  );
}

// Whether the physical light lights the hemisphere by the sky's irradiance: once the sky was projected.
fn is_sky_irradiance(state: Lighting) -> bool {
  return is_lighting_physical(state) && state.sky_harmonics[0].w > 0.5;
}

// The sky's irradiance over π at a world normal, at the light's scale: what a white surface facing it shows of the
// whole sky, unoccluded.
fn sky_irradiance_at(state: Lighting, normal: vec3<f32>) -> vec3<f32> {
  let basis: array<f32, 9> = sky_harmonics_basis(normal);
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var term: u32 = 0u; term < 9u; term++) {
    sum += state.sky_harmonics[term].rgb * basis[term];
  }

  return max(sum, vec3<f32>(0.0)) * light_scale(state);
}
