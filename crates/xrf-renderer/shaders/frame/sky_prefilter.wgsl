#import "common/lighting"
#import "common/sky_box"
#import "common/sky_clouds"

// The sky's light as the frame draws it (`sky_light`: the cubes through the half box, the rim under the fold, the sky's
// colour and the engine's factor, the clouds over it, as light in the engine's units, which a reader gives the light's scale, since
// the cube outlasts the frame it was drawn in), prefiltered by roughness into a cube's mips for what a reflection meets
// nothing in front of: the first mip the sky itself, each after it the sky blurred by the GGX lobe of a rougher surface,
// its roughness the mip over the last mip, so a reflection reads the cube at the mip its roughness names. The cube is
// indexed by a direction in renderer space, the sky's own turn already taken, so a reader passes its reflected direction
// as it is.

// Both keyframes' sky cubes and clouds and their samplers, the mips filled before this one, the lighting, the exposure
// whose scale the sky is drawn on, the mip's settings and the mip's six faces.
#import "generated/frame/sky_prefilter"
#import "common/math"

// The `index`th of `count` points of the Hammersley set, well spread over the unit square.
fn hammersley(index: u32, count: u32) -> vec2<f32> {
  return vec2<f32>(f32(index) / f32(count), f32(reverseBits(index)) * 2.3283064365386963e-10);
}

// A microfacet normal drawn from the GGX distribution about `normal` at a roughness `alpha`, by two uniform numbers.
fn ggx_normal(numbers: vec2<f32>, alpha: f32, normal: vec3<f32>) -> vec3<f32> {
  let angle: f32 = 2.0 * PI * numbers.x;
  let cos_theta: f32 = sqrt((1.0 - numbers.y) / (1.0 + (alpha * alpha - 1.0) * numbers.y));
  let sin_theta: f32 = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
  let local: vec3<f32> = vec3<f32>(sin_theta * cos(angle), sin_theta * sin(angle), cos_theta);
  let up: vec3<f32> = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(normal.z) < 0.999);
  let tangent: vec3<f32> = normalize(cross(up, normal));
  let bitangent: vec3<f32> = cross(normal, tangent);

  return normalize(tangent * local.x + bitangent * local.y + normal * local.z);
}

// The GGX distribution of normals at a cosine to the surface's normal, for a roughness `alpha`.
fn ggx_distribution(cosine: f32, alpha: f32) -> f32 {
  let alpha_squared: f32 = alpha * alpha;
  let d: f32 = cosine * cosine * (alpha_squared - 1.0) + 1.0;

  return alpha_squared / (PI * d * d);
}

// The sky blurred over the GGX lobe of `roughness` about `direction`, the view and the normal taken along it as the
// split-sum prefilter does: each sample read from the mips before this one at the mip its share of the sphere names
// (filtered importance sampling), so few samples leave no speckle.
fn blurred_sky(direction: vec3<f32>, roughness: f32, samples: u32, mips_before: f32) -> vec3<f32> {
  let alpha: f32 = max(roughness * roughness, 1e-3);
  let first: f32 = settings.shape.x;
  // The sphere a texel of the first mip covers.
  let texel_angle: f32 = 4.0 * PI / (6.0 * first * first);
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;

  for (var index: u32 = 0u; index < samples; index++) {
    let microfacet: vec3<f32> = ggx_normal(hammersley(index, samples), alpha, direction);
    let facing: f32 = dot(direction, microfacet);
    let sampled: vec3<f32> = 2.0 * facing * microfacet - direction;
    let lit: f32 = dot(direction, sampled);

    if (lit <= 0.0) {
      continue;
    }

    // The sample's share of the sphere, from the lobe's density about the view's reflection.
    let density: f32 = ggx_distribution(facing, alpha) / 4.0;
    let sample_angle: f32 = 1.0 / (f32(samples) * density + 1e-6);
    let mip: f32 = clamp(0.5 * log2(sample_angle / texel_angle) + 1.0, 0.0, mips_before - 1.0);

    sum += textureSampleLevel(source, sky_clamp, sampled, mip).rgb * lit;
    weight += lit;
  }

  return sum / max(weight, 1e-4);
}

@compute @workgroup_size(8, 8, 1)
fn cs_prefilter(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: u32 = u32(settings.level.w);

  if (any(id.xy >= vec2<u32>(size)) || id.z >= 6u) {
    return;
  }

  let direction: vec3<f32> = cube_face_direction(id.z, (vec2<f32>(id.xy) + 0.5) / f32(size));
  let level: u32 = u32(settings.level.x);
  var colour: vec3<f32>;

  if (level == 0u) {
    // At the pictures' share, as the frame shows its sky, so a reflection of it shows what the sky does; at the
    // anchored levels at the hemisphere the sky lights the diffuse by (`Lighting::sky_reflected`), a mirror of the
    // seen sky reading the screen's own.
    colour = sky_light(direction, lighting, sky_cube_0, sky_cube_1, sky_clamp, sky_clouds_0, sky_clouds_1, sky_repeat,
      picture_exposure(lighting, exposure)) * picture_share(lighting) * lighting.sky_reflected.rgb;
  } else {
    colour = blurred_sky(direction, settings.level.y, u32(settings.level.z), f32(level));
  }

  // The sky stands at no distance; a blurred mip's alpha is unread, the first mip's being the surroundings' distances.
  textureStore(faces, vec2<i32>(id.xy), i32(id.z), vec4<f32>(colour, 0.0));
}
