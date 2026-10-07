#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/contact_march"

// Contact shadows towards the sun: from each drawn pixel facing it, a ray marched over the frame's depth as
// `common/contact_march` marches it, as long as the settings make it. The sunlight kept, in red.

#import "generated/frame/contact_shadows"

@fragment
fn fs_contact_shadows(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return vec4<f32>(1.0);
  }

  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let to_sun: vec3<f32> = contact.to_sun.xyz;

  // A surface facing away is unlit by the sun already.
  if (dot(normal, to_sun) <= 0.0) {
    return vec4<f32>(1.0);
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let plane_normal: vec3<f32> = contact_plane_normal(depth_target, texel, in.clip.xy, position);

  // Every hit inside the thickness whole and every step taken, as the sun's shadows were tuned.
  return vec4<f32>(
    contact_lit(depth_target, contact, in.clip.xy, position, plane_normal, to_sun, contact.length, 0.0, 0.0)
  );
}
