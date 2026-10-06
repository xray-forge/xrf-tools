#import "common/sky_box"
#import "common/water_surface"
#import "generated/static/water"

// Water as OpenXRay's `water.vs`, `water.ps` and `waterd.ps` draw it, and Anomaly's programs over them: the wave
// lifting the surface, two scrolling normal layers bending the sky's reflection, the fresnel mixing it with the base by
// the base's alpha, lit per vertex by the hemisphere, the sun and the ambient. Soft water fades by the depth behind it,
// darkens with it towards `water_intensity`, lays foam in the shallows and is fogged. Drawn over the lit frame as it
// shows, which `water.ps` writes without the tonemap, and the distortion it causes into a target of its own.

@vertex
fn vs_water(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> WaterVarying {
  return water_vertex(vertex_index, instance_index, water);
}

// Both keyframes' skies along a world direction, as water reflecting them reads the cubes: unturned by the sky's
// rotation, which turns only the sky box.
fn sky_cubes(direction: vec3<f32>) -> vec3<f32> {
  let lookup: vec3<f32> = cube_lookup(direction);

  return mix(
    textureSampleLevel(sky_cube_0, sky_clamp, lookup, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, lookup, 0.0).rgb,
    lighting.sky.w,
  );
}

@fragment
fn fs_water(in: WaterVarying) -> WaterOutput {
  let fragment: WaterFragment = read_water(in, water, lighting, depth_target, nearest_water);
  let flags: u32 = fragment.flags;
  let base: vec4<f32> = fragment.base;
  let reflected: vec3<f32> = fragment.reflected;
  let to_point: vec3<f32> = fragment.to_point;
  let light: vec3<f32> = fragment.light;
  var lit: vec3<f32>;
  var plain_alpha: f32;

  if ((flags & SURFACE_IS_ANOMALY_WATER) != 0u) {
    // Anomaly's `water.ps`: the whole sky mixed with the base by its alpha, the sun's highlight over it, by the
    // switches its program defines; its fresnel is taken of the reflection before the sky's remapping.
    let power: f32 = pow(saturate(dot(reflected, to_point)), 9.0);
    let sky: vec3<f32> = sky_cubes(vec3<f32>(reflected.x, reflected.y * 2.0 - 1.0, reflected.z)) * water.reflection;
    let albedo: vec3<f32> = select(base.rgb, base.rgb * light, (flags & SURFACE_IS_TRANSPARENT) != 0u);
    var color: vec3<f32> = select(albedo, mix(sky, albedo, base.a), (flags & SURFACE_IS_REFLECTING) != 0u);

    if ((flags & SURFACE_IS_SPECULAR) != 0u) {
      // `specular_phong(v2point, Nw, L_sun_dir_w) * 4`.
      let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
      let sun: vec3<f32> = -(rotation * lighting.to_sun.xyz);

      color += lighting.sun.rgb * pow(abs(dot(normalize(to_point + sun), fragment.surface_normal)), 256.0) * 4.0;
    }

    lit = color * light * 2.0;
    plain_alpha = 0.55 + power * 0.25;
  } else {
    // OpenXRay's `water.ps`: the sky squared and doubled, a share of it by the fresnel, mixed with the base by its
    // alpha.
    let power: f32 = pow(saturate(dot(fragment.remapped, to_point)), 9.0);
    let sky: vec3<f32> = sky_cubes(fragment.remapped);
    let amount: f32 = (0.15 + power * 0.25) * water.reflection;

    lit = mix(sky * sky * 2.0 * amount, base.rgb, base.a) * light * 2.0;
    plain_alpha = 0.75 + power * 0.25;
  }

  // `NEED_SOFT_WATER` and `USE_SOFT_WATER`: the depth behind the surface, which fades, darkens and foams it.
  let depth: f32 = fragment.depth;
  let deepened: vec3<f32> = mix(vec3<f32>(water.intensity * 0.1), lit, plain_alpha);
  let faded: f32 = max(1.0 - exp(depth * -4.0), min(plain_alpha, saturate(depth)));
  let shallow: f32 = saturate(-depth * dot(fragment.normal, to_point));
  let is_foamed: bool = (flags & SURFACE_IS_ANOMALY_WATER) == 0u || (flags & SURFACE_IS_FOAMED) != 0u;
  let foamed: f32 = smoothstep(0.025, 0.05, shallow) * (1.0 - smoothstep(0.075, 0.1, shallow)) * fragment.foam.a *
    camera.switches.x * f32(is_foamed);
  let soft_color: vec3<f32> = mix(mix(deepened, fragment.foam.rgb * water.intensity, foamed), lit, 1.0 - water.soft);
  let soft_alpha: f32 = mix(mix(faded, fragment.foam.a, foamed), plain_alpha, 1.0 - water.soft);
  let is_soft: bool = (flags & SURFACE_IS_SOFT_WATER) != 0u;
  let color: vec3<f32> = select(lit, soft_color, is_soft);
  let seen: f32 = 1.0 - fragment.fog;
  // Plain `water` is written whole, `blend(false)`; soft water is faded by the fog twice over, as its alpha is.
  let alpha: f32 = select(1.0, soft_alpha * seen * seen, is_soft);
  let finished: vec3<f32> = mix(color, lighting.fog_color.rgb, fragment.fog);
  var out: WaterOutput;

  out.color = vec4<f32>(select(base.rgb, finished, lighting.params.y > 0.5), alpha);
  out.distortion = write_distortion(fragment, water);

  return out;
}
