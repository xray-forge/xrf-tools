#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/contact_march"

// Contact shadows towards the sun at the frame's size or half of it (`stride`): from each drawn pixel facing it, or the
// first of each two by two, a ray marched over the frame's depth as `common/contact_march` marches it, as long as the
// settings make it. The sunlight kept, then the pixel's distance along the view, by which the sun brings a half-size
// march up to its own pixels.

#import "generated/frame/contact_shadows"

// What a pixel stands for where nothing was drawn: lit, as far as a half float reaches.
const NOTHING_DRAWN: vec4<f32> = vec4<f32>(1.0, 65504.0, 0.0, 0.0);

@fragment
fn fs_contact_shadows(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<i32> = vec2<i32>(textureDimensions(depth_target));
  let texel: vec2<i32> = min(vec2<i32>(in.clip.xy) * i32(contact.stride), size - 1);
  let pixel: vec2<f32> = vec2<f32>(texel) + 0.5;
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return NOTHING_DRAWN;
  }

  let position: vec3<f32> = camera_view_position(pixel, depth);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let to_sun: vec3<f32> = contact.to_sun.xyz;

  // A surface facing away is unlit by the sun already.
  if (dot(normal, to_sun) <= 0.0) {
    return vec4<f32>(1.0, -position.z, 0.0, 0.0);
  }

  let plane_normal: vec3<f32> = contact_plane_normal(depth_target, texel, pixel, position);

  // Every hit inside the thickness whole and every step taken, as the sun's shadows were tuned.
  let lit: f32 = contact_lit(depth_target, contact, pixel, position, plane_normal, to_sun, contact.length, 0.0, 0.0);

  return vec4<f32>(lit, -position.z, 0.0, 0.0);
}
