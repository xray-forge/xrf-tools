#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/fullscreen"
#import "common/sun_shadow"
#import "common/occlusion"

// `accum_sun`: the sun at every drawn pixel, `Ldynamic_color * plight_infinity(m, P, N, L)`, times how much of it
// reaches the pixel through the cascades' maps, or, traced, what the sun's traced shadow saw there, and past what its
// contact shadows found. Diffuse in colour, specular in alpha, into the light the frame accumulates; and beside it how
// much of the sun reaches the pixel, which combine's glint on a wet surface's water film is shadowed by.

#import "generated/frame/sun"

// What `accum_emissive.ps` writes into a self-lit surface's light.
const EMISSIVE_LIGHT: f32 = 16.0;

// The sun's light at a pixel, and how much of the sun reaches it.
struct SunTargets {
  @location(0) light: vec4<f32>,
  @location(1) seen: vec4<f32>,
};

@fragment
fn fs_sun(in: FullscreenVarying) -> SunTargets {
  return light_sun(in, false);
}

@fragment
fn fs_sun_traced(in: FullscreenVarying) -> SunTargets {
  return light_sun(in, true);
}

// The sun's light at a pixel, its shadow the cascades' or, `is_traced`, the traced shadow's.
fn light_sun(in: FullscreenVarying, is_traced: bool) -> SunTargets {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  let view: u32 = light_view();

  // The lamps' and the overdraw's views keep the sun out of what they count.
  if (depth <= 0.0 || view == LIGHT_VIEW_LAMPS || view == LIGHT_VIEW_OVERDRAW) {
    return SunTargets(vec4<f32>(0.0), vec4<f32>(0.0));
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);

  // The cascades' view: the cascade the pixel reads, from one, none past them all, and how much of the sun it lets
  // through there, self-lit surfaces alike.
  if (view == LIGHT_VIEW_CASCADES) {
    let world: vec3<f32> = (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz + camera.position.xyz;
    let world_normal: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(normal, 0.0)).xyz);
    let cascade: u32 = sun_cascade(shadows, world);
    let order: f32 = select(0.0, f32(cascade) + 1.0, cascade < shadows.count);
    let seen: f32 = sun_shadow(shadow_maps, shadows, world, world_normal, dot(normal, lighting.to_sun.xyz));

    return SunTargets(vec4<f32>(order, seen, 0.0, 0.0), vec4<f32>(0.0));
  }

  // `accum_emissive`: a self-lit surface's light is filled with sixteen, which the local lights then add to.
  if (has_mark(material.a, MARK_EMISSIVE)) {
    return SunTargets(vec4<f32>(EMISSIVE_LIGHT), vec4<f32>(0.0));
  }

  let slice: f32 = material.z;
  // `plight_infinity`: L towards the light, V towards the eye, H halfway.
  let to_light: vec3<f32> = lighting.to_sun.xyz;
  let half_way: vec3<f32> = normalize(to_light - normalize(position));
  let lit: vec4<f32> = textureSampleLevel(
    material_lut,
    lut_sampler,
    vec3<f32>(dot(to_light, normal), dot(half_way, normal), slice),
    0.0
  );
  let world: vec3<f32> = (transpose(camera.view) * vec4<f32>(position, 0.0)).xyz + camera.position.xyz;
  let world_normal: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(normal, 0.0)).xyz);
  // Marched at the frame's size and read where it is, or at half and brought up by depth; a single lit texel stands in
  // where none are drawn.
  var contact: f32 = 1.0;

  if (all(textureDimensions(contact_shadows) >= vec2<u32>(camera.viewport.xy))) {
    contact = textureLoad(contact_shadows, vec2<i32>(in.clip.xy), 0).x;
  } else {
    contact = upsampled_occlusion(contact_shadows, floor(in.clip.xy), -position.z);
  }
  // Terrain's and a marched steep surface's own sun term is what the enhanced parallax keeps from its hollows, one
  // where none marches it.
  let relief: f32 = select(1.0, material.y, has_mark(material.a, MARK_TERRAIN) || has_mark(material.a, MARK_RELIEF));
  var seen: f32;

  if (is_traced) {
    seen = textureLoad(sun_traced, texel, 0).x;
  } else if (has_mark(material.a, MARK_PLANT)) {
    seen = plant_sun_shadow(shadow_maps, shadows, world, world_normal, dot(normal, to_light));
  } else {
    seen = sun_shadow(shadow_maps, shadows, world, world_normal, dot(normal, to_light));
  }

  let shadow: f32 = seen * contact * relief;

  // Foliage passes the sun through from behind: by how directly the eye looks into it past the leaf, tinted by the
  // sun's colour as far as the flora's colour share says.
  var through: vec3<f32> = vec3<f32>(0.0);

  if (lighting.flora.x > 0.5 && has_mark(material.a, MARK_FLORA)) {
    let to_eye: vec3<f32> = -normalize(position);
    let amount: f32 = saturate(dot(to_eye, -(to_light + normal))) * lighting.flora.y;

    through = amount * mix(vec3<f32>(1.0), lighting.sun.rgb, lighting.flora.z);
  }

  return SunTargets(vec4<f32>(lighting.sun.rgb * lit.x + through, lighting.sun.w * lit.y) * shadow, vec4<f32>(shadow));
}
