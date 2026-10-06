// A slice's visibility as a mask of 32 bits, after Therrien, Levesque and Gilet's "Screen Space Indirect Lighting with
// Visibility Bitmask" (2023): every sample an occluder of a thickness, covering the angles between the point the depth
// shows and the point that far behind it, each bit set where an occluder covers it. The slice's set-up is GTAO's
// (Jimenez et al., "Practical Real-Time Strategies for Accurate Indirect Occlusion", 2016). The bits here are spread
// evenly over the slice's cosine-weighted measure rather than its angles, so a clear mask's share is the visibility
// GTAO integrates and the bits along the tangent plane weigh least. Nothing is bound: a search passes its samples in,
// and a caller gathering light weighs each sample by the bits it sets that were still clear.

const BITMASK_BITS: u32 = 32u;
const BITMASK_PI: f32 = 3.14159265;
const BITMASK_HALF_PI: f32 = 1.57079633;

// One slice through the view vector: where the normal lies in it, and how much its visible half-circle weighs.
struct BitmaskSlice {
  // The angle from the view vector to the normal projected into the slice, positive towards the slice's direction.
  normal_angle: f32,
  normal_cos: f32,
  normal_sin: f32,
  // The cosine-weighted measure from the lower tangent to the view vector.
  lower: f32,
  // The whole half-circle's cosine-weighted measure.
  measure: f32,
  // What the slice weighs among the others: its projected normal's length times its measure.
  weight: f32,
};

// Arc cosine to within 7e-5, Abramowitz and Stegun's 4.4.45 mirrored below zero.
fn bitmask_acos(x: f32) -> f32 {
  let a: f32 = min(abs(x), 1.0);
  let r: f32 = sqrt(1.0 - a) * (1.5707288 + a * (-0.2121144 + a * (0.0742610 + a * -0.0187293)));

  return select(r, BITMASK_PI - r, x < 0.0);
}

// The slice along a view space `direction` across the view, at a point seen along `to_camera` with a `normal`.
fn bitmask_slice(normal: vec3<f32>, to_camera: vec3<f32>, direction: vec3<f32>) -> BitmaskSlice {
  let across: vec3<f32> = direction - to_camera * dot(direction, to_camera);
  let axis: vec3<f32> = normalize(cross(across, to_camera));
  let projected: vec3<f32> = normal - axis * dot(normal, axis);
  let projected_length: f32 = length(projected);
  let facing: f32 = saturate(dot(projected, to_camera) / max(projected_length, 1e-4));
  let angle: f32 = sign(dot(across, projected)) * bitmask_acos(facing);
  let normal_cos: f32 = cos(angle);
  let normal_sin: f32 = sin(angle);
  let lower: f32 = (2.0 * normal_cos + (2.0 * angle - BITMASK_PI) * normal_sin) * 0.25;
  let measure: f32 = normal_cos + angle * normal_sin;

  return BitmaskSlice(angle, normal_cos, normal_sin, lower, measure, projected_length * measure);
}

// The share of the slice's cosine-weighted measure from its lower tangent up to an angle from the view vector, from
// zero to one: `∫ cos(t - n) |sin t| dt`, which from the view vector is `(cos n + 2θ sin n - cos(2θ - n)) / 4`.
fn bitmask_share(slice: BitmaskSlice, angle: f32) -> f32 {
  let n: f32 = slice.normal_angle;
  let held: f32 = clamp(angle, n - BITMASK_HALF_PI, n + BITMASK_HALF_PI);
  let from_view: f32 = (slice.normal_cos + 2.0 * held * slice.normal_sin - cos(2.0 * held - n)) * 0.25;

  return saturate((slice.lower + sign(held) * from_view) / slice.measure);
}

// The bits whose centres lie between two shares.
fn bitmask_span(low: f32, high: f32) -> u32 {
  let first: u32 = u32(clamp(round(low * f32(BITMASK_BITS)), 0.0, f32(BITMASK_BITS)));
  let last: u32 = u32(clamp(round(high * f32(BITMASK_BITS)), 0.0, f32(BITMASK_BITS)));

  if (last <= first) {
    return 0u;
  }

  let count: u32 = last - first;

  return select(0xffffffffu >> (BITMASK_BITS - count), 0xffffffffu, count >= BITMASK_BITS) << first;
}

// The bits an occluder covers: from the point the depth shows to the point its thickness behind, each an offset from
// the shaded point in view space, on the side of the view vector `side` names (one along the slice's direction, minus
// one against it).
fn bitmask_occluder(slice: BitmaskSlice, to_camera: vec3<f32>, front: vec3<f32>, back: vec3<f32>, side: f32) -> u32 {
  let front_share: f32 = bitmask_share(slice, side * bitmask_acos(dot(normalize(front), to_camera)));
  let back_share: f32 = bitmask_share(slice, side * bitmask_acos(dot(normalize(back), to_camera)));

  return bitmask_span(min(front_share, back_share), max(front_share, back_share));
}

// The share of the slice's cosine-weighted measure a mask leaves clear.
fn bitmask_visibility(mask: u32) -> f32 {
  return 1.0 - f32(countOneBits(mask)) / f32(BITMASK_BITS);
}
