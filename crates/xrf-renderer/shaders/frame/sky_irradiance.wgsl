#import "common/lighting"
#import "common/sky_box"
#import "common/sky_clouds"
#import "common/sky_harmonics"

// The sky's irradiance for the physical light: the sky as the frame draws it (`sky_light`, clouds and all, in the
// engine's units) projected over the whole sphere into order-2 spherical harmonics, then convolved by the cosine, so a
// normal's nine products give the irradiance over π that lights a surface facing it (Ramamoorthi and Hanrahan, "An
// Efficient Representation for Irradiance Environment Maps", SIGGRAPH 2001). One workgroup samples a spherical
// Fibonacci set and sums it.

// Both keyframes' sky cubes and clouds and their samplers, the lighting, the exposure whose scale the sky is drawn on,
// and the harmonics written.
#import "generated/frame/sky_irradiance"
#import "common/math"

// Invocations of the workgroup (`SkyIrradiancePass::WORKGROUP`) and directions each samples.
const INVOCATIONS: u32 = 64u;
const SAMPLES_EACH: u32 = 16u;
// The golden angle, which turns each Fibonacci direction from the last.
const GOLDEN_ANGLE: f32 = 2.39996323;
// Each harmonic band's cosine convolution over π: `π`, `2π / 3` and `π / 4` of Ramamoorthi and Hanrahan, over π.
const BAND_0: f32 = 1.0;
const BAND_1: f32 = 2.0 / 3.0;
const BAND_2: f32 = 0.25;

var<workgroup> sums: array<array<vec3<f32>, 9>, 64>;

@compute @workgroup_size(64)
fn cs_irradiance(@builtin(local_invocation_index) index: u32) {
  let count: u32 = INVOCATIONS * SAMPLES_EACH;
  let scale: f32 = picture_exposure(lighting, exposure);
  var sum: array<vec3<f32>, 9>;

  for (var sample: u32 = 0u; sample < SAMPLES_EACH; sample++) {
    let at: u32 = index * SAMPLES_EACH + sample;
    let height: f32 = 1.0 - (f32(at) + 0.5) * 2.0 / f32(count);
    let around: f32 = sqrt(max(1.0 - height * height, 0.0));
    let angle: f32 = f32(at) * GOLDEN_ANGLE;
    let direction: vec3<f32> = vec3<f32>(around * cos(angle), height, around * sin(angle));
    let light: vec3<f32> = sky_light(direction, lighting, sky_cube_0, sky_cube_1, sky_clamp, sky_clouds_0, sky_clouds_1,
      sky_repeat, scale);
    let basis: array<f32, 9> = sky_harmonics_basis(direction);

    for (var term: u32 = 0u; term < 9u; term++) {
      sum[term] += light * basis[term];
    }
  }

  sums[index] = sum;
  workgroupBarrier();

  if (index != 0u) {
    return;
  }

  // Each direction stands for an equal share of the sphere.
  let solid_angle: f32 = 4.0 * PI / f32(count);
  var bands: array<f32, 9> = array<f32, 9>(BAND_0, BAND_1, BAND_1, BAND_1, BAND_2, BAND_2, BAND_2, BAND_2, BAND_2);

  for (var term: u32 = 0u; term < 9u; term++) {
    var total: vec3<f32> = vec3<f32>(0.0);

    for (var invocation: u32 = 0u; invocation < INVOCATIONS; invocation++) {
      total += sums[invocation][term];
    }

    // The first's `w` says the sky was projected.
    harmonics.coefficients[term] = vec4<f32>(total * solid_angle * bands[term], select(0.0, 1.0, term == 0u));
  }
}
