#import "common/math"

// One microfacet model for every path that shades a surface by light from a direction: GGX's distribution (Walter et
// al., "Microfacet Models for Refraction through Rough Surfaces", EGSR 2007), Smith's height-correlated visibility
// (Heitz, "Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs", JCGT 2014), Schlick's Fresnel and
// Lambert's diffuse; the energy a single scattering loses at a rough lobe given back from the split-sum table
// (Fdez-Agüera, "A Multiple-Scattering Microfacet Model for Real-Time Image-based Lighting", JCGT 2019, in Filament's
// form). A roughness is the perceptual one, its lobe's alpha its square.

// The least cosine of the view to a surface's normal the lobe is taken at: the G-buffer's normals turn a few faces past
// the view at silhouettes, where none would leave them black.
const BRDF_LEAST_FACING: f32 = 1e-4;

// The GGX lobe's alpha for a roughness: its square, as the Disney and Unreal convention has it, so a roughness of
// 0.02 is a lobe a fiftieth of a degree wide and not a degree, which read as the whole puddle jittering.
fn lobe_alpha(roughness: f32) -> f32 {
  return roughness * roughness;
}

// GGX's distribution of normals at the cosine of the normal to the half vector, for a lobe of `alpha`.
fn ggx_distribution(along: f32, alpha: f32) -> f32 {
  let alpha_squared: f32 = alpha * alpha;
  let spread: f32 = along * along * (alpha_squared - 1.0) + 1.0;

  return alpha_squared / (PI * spread * spread);
}

// Smith's height-correlated masking and shadowing over `4 N·L N·V`, at the cosines of the light (`lit`) and the view
// (`facing`) to the normal.
fn smith_visibility(lit: f32, facing: f32, alpha: f32) -> f32 {
  let alpha_squared: f32 = alpha * alpha;

  return 0.5 / (lit * sqrt(facing * facing * (1.0 - alpha_squared) + alpha_squared) +
    facing * sqrt(lit * lit * (1.0 - alpha_squared) + alpha_squared));
}

// Schlick's Fresnel over a reflectance head on, at the cosine of the light to the half vector.
fn schlick_fresnel(head_on: vec3<f32>, cosine: f32) -> vec3<f32> {
  return head_on + (1.0 - head_on) * pow(1.0 - saturate(cosine), 5.0);
}

// The split-sum's scale and bias at a roughness and the cosine of the view to the normal: Karis's fit of the DFG term,
// what share of a uniform environment a surface reflects as `head_on * scale + bias`.
fn split_sum(roughness: f32, facing: f32) -> vec2<f32> {
  let r: vec4<f32> = roughness * vec4<f32>(-1.0, -0.0275, -0.572, 0.022) + vec4<f32>(1.0, 0.0425, 1.04, -0.04);
  let a004: f32 = min(r.x * r.x, exp2(-9.28 * saturate(facing))) * r.x + r.y;

  return vec2<f32>(-1.04, 1.04) * a004 + r.zw;
}

// What share of the light arriving along a surface's reflection it sends to the eye, at a cosine of the view to its
// normal: the split-sum environment BRDF over its reflectance head on and its roughness, which holds the Fresnel's rise
// at grazing angles and its fall with roughness both.
fn environment_brdf(head_on: vec3<f32>, roughness: f32, facing: f32) -> vec3<f32> {
  let ab: vec2<f32> = split_sum(roughness, facing);

  return head_on * ab.x + ab.y;
}

// The texels of the split-sum table (`BrdfTable`) each way, the cosine of the view to the normal across and the
// roughness down, each at its texel's centre.
const BRDF_TABLE_SIZE: f32 = 32.0;

// The split-sum table's scale and bias at a roughness and the cosine of the view to the normal, read between its four
// nearest texels: the table holds 32-bit floats, which no sampler filters everywhere.
fn brdf_table_split(table: texture_2d<f32>, roughness: f32, facing: f32) -> vec2<f32> {
  let at: vec2<f32> = clamp(vec2<f32>(facing, roughness) * BRDF_TABLE_SIZE - 0.5, vec2<f32>(0.0),
    vec2<f32>(BRDF_TABLE_SIZE - 1.0));
  let corner: vec2<i32> = vec2<i32>(floor(at));
  let next: vec2<i32> = min(corner + 1, vec2<i32>(i32(BRDF_TABLE_SIZE) - 1));
  let f: vec2<f32> = at - floor(at);
  let top: vec2<f32> = mix(textureLoad(table, corner, 0).xy, textureLoad(table, vec2<i32>(next.x, corner.y), 0).xy, f.x);
  let bottom: vec2<f32> =
    mix(textureLoad(table, vec2<i32>(corner.x, next.y), 0).xy, textureLoad(table, next, 0).xy, f.x);

  return mix(top, bottom, f.y);
}

// What a single scattering's lobe is multiplied by to hold the energy multiple scattering would return, from the
// lobe's directional albedo for white (`white`, the table's scale and bias summed): one over it, scaled by the
// reflectance head on, so a rough metal is no darker than a smooth one.
fn multiple_scattering(head_on: vec3<f32>, white: f32) -> vec3<f32> {
  return 1.0 + head_on * (1.0 / max(white, 1e-3) - 1.0);
}

// A sphere light as GGX's specular takes it from a single direction (Karis, "Real Shading in Unreal Engine 4",
// SIGGRAPH 2013): the direction to the sphere's point nearest the view's reflection (`reflected`) in place of its
// centre's, and the share of the lobe kept, `(α / α′)²`, as the sphere's angular size widens its alpha to
// `α′ = α + r / 3d`. Against the sphere's disc taken as the many point lights it is, a rough lobe keeps within a
// quarter under it; `r / 2d`, the half vector's turn, kept a third under. `to_centre` runs from the surface to the
// light's centre `d` away, `radius` is the sphere's `r`.
struct SphereLight {
  to_light: vec3<f32>,
  kept: f32,
};

fn sphere_light(to_centre: vec3<f32>, radius: f32, reflected: vec3<f32>, roughness: f32) -> SphereLight {
  let across: vec3<f32> = dot(to_centre, reflected) * reflected - to_centre;
  let nearest: vec3<f32> = to_centre + across * saturate(radius / max(length(across), 1e-6));
  let alpha: f32 = lobe_alpha(roughness);
  let widened: f32 = saturate(alpha + radius / (3.0 * max(length(to_centre), 1e-4)));

  return SphereLight(normalize(nearest), alpha * alpha / max(widened * widened, 1e-8));
}

// The light a surface sends the eye from a light along `to_light`, per unit of the light's colour as the engine lights
// a surface by it (its colour times the cosine on a white surface, the irradiance over π): the diffuse's cosine, which
// the caller multiplies by the albedo, and the specular, GGX's lobe over the reflectance head on and roughness times
// π, the cosine and the multiple scattering's gift from the lobe's directional albedo for white (`white`, from
// `brdf_table_split` at the view's cosine), taken towards a sphere light's nearest point (`sphere`; its centre's
// direction and all of the lobe for a point). `normal`, `to_eye` and the directions share a space.
struct DirectLight {
  diffuse: f32,
  specular: vec3<f32>,
};

fn direct_light(normal: vec3<f32>, to_eye: vec3<f32>, to_light: vec3<f32>, sphere: SphereLight, roughness: f32,
  head_on: vec3<f32>, white: f32) -> DirectLight {
  let lit: f32 = dot(normal, to_light);

  if (lit <= 0.0) {
    return DirectLight(0.0, vec3<f32>(0.0));
  }

  let glinting: f32 = saturate(dot(normal, sphere.to_light));
  let facing: f32 = max(dot(normal, to_eye), BRDF_LEAST_FACING);
  let half_way: vec3<f32> = normalize(sphere.to_light + to_eye);
  let alpha: f32 = lobe_alpha(roughness);
  let lobe: f32 = ggx_distribution(saturate(dot(normal, half_way)), alpha) *
    smith_visibility(max(glinting, BRDF_LEAST_FACING), facing, alpha) * sphere.kept;
  let fresnel: vec3<f32> = schlick_fresnel(head_on, dot(to_eye, half_way));

  return DirectLight(lit, PI * lobe * glinting * fresnel * multiple_scattering(head_on, white));
}
