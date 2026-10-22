#import "common/camera"
#import "common/lighting"
#import "common/scene_light"
#import "common/sun_shadow"
#import "common/light_clusters"
#import "common/local_lights"
#import "common/froxel"

// The enhanced volumetric lighting's froxels filled: at a point jittered along each froxel's slice, the light the air
// scatters towards the camera, from the sun through its cascades and from every local light binned where the froxel
// stands through its shadow, by the phase; and how thick the air is, its own haze and the enhanced fog's height fog.
// Kept over frames: the froxel's history read where its point stood the frame before.

#import "generated/frame/froxel_inject"

// Scattering a metre of clear air holds for the sun at the weather's full shafts, so a sunlit ray of 40 m adds what the
// engine's march adds over a whole ray; and how much the enhanced fog's height fog thickens it at its densest.
const FROXEL_SUN: f32 = 1.0 / 40.0;
const FROXEL_FOG: f32 = 4.0;
// Scattering a metre holds for the local lights: a faint glow about a fire or a lamp, its shaft where it is shadowed.
const FROXEL_LAMP: f32 = 0.003;
// How thick a metre of the air is, beside what scatters the sun.
const FROXEL_HAZE: f32 = 0.004;
// How much of the history a froxel keeps.
const FROXEL_HISTORY: f32 = 0.9;

@compute @workgroup_size(8, 8, 1)
fn cs_froxel_inject(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec3<u32> = vec3<u32>(froxels.size.xyz);

  if (any(id >= size)) {
    return;
  }

  let screen: vec2<f32> = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size.xy);
  let ndc: vec2<f32> = vec2<f32>(screen.x * 2.0 - 1.0, 1.0 - screen.y * 2.0);
  let t: f32 = (f32(id.z) + froxels.params.z) / f32(size.z);
  let depth: f32 = max(froxel_depth(froxels, t), 0.05);
  let ray: vec4<f32> = camera.inverse_projection * vec4<f32>(ndc, 1.0, 1.0);
  let along: vec3<f32> = ray.xyz / ray.w;
  let position: vec3<f32> = along * (depth / max(-along.z, 1e-6));
  let world: vec3<f32> = (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz + camera.position.xyz;
  let to_eye: vec3<f32> = -normalize(position);
  // Half the froxel's extent, along the view (its slice's depth) or across it, whichever is wider.
  let spread: f32 = 0.5 * max(2.0 * froxels.size.w * t / f32(size.z),
    depth * 2.0 / (camera.projection[1][1] * f32(size.y)));
  let intensity: f32 = froxels.params.x;
  let g: f32 = froxels.params.y;
  let fog: f32 = 1.0 + height_fog_share(lighting, world.y) * lighting.height_fog.y * FROXEL_FOG;

  // The sun, scattered as thickly as the weather's shafts, through one tap of its cascades. At the authored levels the
  // light the air scatters is gathered as the engine gathers its shafts, the authored colours by the densities as
  // display values in the engine's units, which the composite adds to the frame's display values
  // (`fs_sun_shafts_over`); elsewhere as light.
  let is_display: bool = is_light_summed(lighting);
  let sun_density: f32 = lighting.shafts.x * FROXEL_SUN * fog * intensity;
  var scattered: vec3<f32> = vec3<f32>(0.0);

  if (sun_density > 0.0) {
    let sunlit: f32 = sun_shadow_tap(shadow_maps, shadows, world);
    let sun: vec3<f32> = select(lighting.sun.rgb * authored_amount(sun_density),
      lighting.summed_light.yzw * sun_density, is_display);

    scattered += sun * sunlit * froxel_phase(g, dot(-lighting.to_sun.xyz, to_eye));
  }

  // Every local light binned where the froxel stands that the level gives shafts (`LightRecord::scatter`), through its
  // shadow, as the engine draws only those lights' shafts: an unshadowed lamp would scatter through the walls between
  // it and the air it lights, and a campfire or a wall lamp would hang a ball of glowing fog about itself.
  if (lights.count > 0u) {
    let cluster: u32 = light_cluster(lights, screen, depth);
    let lamp_density: f32 = FROXEL_LAMP * fog * intensity;

    for (var index: u32 = 0u; index < counts[cluster]; index++) {
      let record: LightRecord = records[items[cluster * LIGHT_CLUSTER_CAPACITY + index]];
      let to_point: vec3<f32> = position - record.position.xyz;
      let falloff: f32 = light_falloff(record, to_point);

      if (record.scatter.x <= 0.0 || falloff <= 0.0 || is_outside_cone(record, to_point)) {
        continue;
      }

      let lit: f32 = light_share(record, to_point, are_levels_physical(), spread) *
        light_shadow(shadow_atlas, shadow_sampler, record, to_point, vec3<f32>(0.0), record.axis.w > -1.0, false);

      var lamp: vec3<f32> = record.color.rgb * authored_amount(lamp_density);

      if (is_display) {
        lamp = srgb_light_authored(record.color.rgb / authored_local_scale(lighting)) * lamp_density;
      }

      scattered += lamp * lit * froxel_phase(g, dot(normalize(to_point), to_eye));
    }
  }

  let thickness: f32 = sun_density + FROXEL_HAZE * fog * intensity;
  var current: vec4<f32> = vec4<f32>(scattered, thickness);

  // Where the point stood the frame before, through the last unjittered view: its froxel there, as far along.
  let previous: vec4<f32> = camera.motion_previous * vec4<f32>(world, 1.0);

  if (froxels.params.w > 0.5 && previous.w > 0.05) {
    let previous_ndc: vec2<f32> = previous.xy / previous.w;
    let previous_screen: vec2<f32> = vec2<f32>(previous_ndc.x * 0.5 + 0.5, 0.5 - previous_ndc.y * 0.5);
    let previous_share: f32 = froxel_share(froxels, previous.w);

    if (all(previous_screen >= vec2<f32>(0.0)) && all(previous_screen <= vec2<f32>(1.0)) && previous_share < 1.0) {
      let kept: vec4<f32> = textureSampleLevel(froxel_history, froxel_sampler,
        vec3<f32>(previous_screen, previous_share), 0.0);

      // Light kept at the last frame's pre-exposure is brought to this one's; display values have no exposure.
      let carried: f32 = select(history_scale(lighting), 1.0, is_display);

      current = mix(current, vec4<f32>(kept.rgb * carried, kept.a), FROXEL_HISTORY);
    }
  }

  textureStore(froxel_current, id, current);
}
