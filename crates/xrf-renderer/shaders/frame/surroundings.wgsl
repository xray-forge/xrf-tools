#import "common/camera"
#import "common/scene_light"
#import "common/octahedral"
#import "common/lighting"
#import "common/hmodel"
#import "common/sky_box"
#import "common/sky_clouds"

// One face of the surroundings cube: the level's static scene around the camera, drawn by the face's camera into a
// G-buffer and lit here as the frame lights a surface (`hmodel` over its hemisphere, the sun by the baked sun factor,
// the fog to it), the sky's light where nothing was drawn, into the prefiltered cube's first mip, which the mips after
// it blur by roughness; each texel's metres from the camera in its alpha, nothing for the sky, which a reflection reads
// to aim through the point its ray would meet (`surroundings_direction`). A reflection meeting nothing on the screen
// reads it: behind a bush's twigs it finds the tree line, past the screen's edge the hill, where the sky alone answered
// before.

// The face's G-buffer, the material table, the sky's cubes, clouds and samplers, the lighting, the exposure, the
// face's settings and the cube's first mip.
#import "generated/frame/surroundings"

// The face's G-buffer texels a cube texel averages a side.
const BOX: u32 = 2u;

// What one G-buffer texel of the face comes to as light: the surface lit, fogged to it, and in `w` its metres from the
// camera; the sky's light where none, at no distance.
fn lit_texel(texel: vec2<i32>, direction: vec3<f32>, scale: f32) -> vec4<f32> {
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return vec4<f32>(sky_light(direction, lighting, sky_cube_0, sky_cube_1, sky_clamp, sky_clouds_0, sky_clouds_1,
      sky_repeat, scale), 0.0);
  }

  let stored: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let albedo: vec4<f32> = vec4<f32>(authored_colour(stored.rgb), stored.a);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let normal_world: vec3<f32> = normalize(rotation * normal);
  let to_point_world: vec3<f32> = normalize(rotation * position);
  let height: f32 = camera.position.y + (rotation * position).y;
  // The sun as the baked sun factor lets it, its specular left out: a probe's texel is read blurred.
  let sun: vec3<f32> = lighting.sun.rgb * max(dot(normal_world, settings.to_sun.xyz), 0.0) * material.y;
  let lit: vec3<f32> = hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp,
    albedo, vec4<f32>(sun, 0.0), normal_world, to_point_world, material.z, material.x, vec3<f32>(1.0));
  let fog: f32 = select(0.0, fog_amount_at(lighting, position, height), lighting.fog_color.w > 0.5);

  return vec4<f32>(fog_mixed(lighting, lit, fog_color_at(lighting, position, height), fog), length(position));
}

@compute @workgroup_size(8, 8, 1)
fn cs_resolve(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: u32 = u32(settings.face.y);

  if (any(id.xy >= vec2<u32>(size))) {
    return;
  }

  let face: u32 = u32(settings.face.x);
  let drawn: u32 = u32(settings.face.z);
  let scale: f32 = frame_scale(lighting, exposure);
  let direction: vec3<f32> = cube_face_direction(face, (vec2<f32>(id.xy) + 0.5) / f32(size));
  let first: vec2<i32> = vec2<i32>(id.xy * BOX);
  var sum: vec3<f32> = vec3<f32>(0.0);
  // The distance stored is the nearest the box holds: a thin trunk over the sky keeps its own, not the sky's none.
  var nearest: f32 = 0.0;

  for (var y: u32 = 0u; y < BOX; y++) {
    for (var x: u32 = 0u; x < BOX; x++) {
      let texel: vec2<i32> = min(first + vec2<i32>(i32(x), i32(y)), vec2<i32>(i32(drawn) - 1));
      let lit: vec4<f32> = lit_texel(texel, direction, scale);

      sum += lit.rgb;
      nearest = select(nearest, lit.w, lit.w > 0.0 && (nearest <= 0.0 || lit.w < nearest));
    }
  }

  textureStore(faces, vec2<i32>(id.xy), i32(face), vec4<f32>(sum / f32(BOX * BOX), nearest));
}
