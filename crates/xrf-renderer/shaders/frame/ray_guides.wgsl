#import "common/camera"
#import "common/scene_light"
#import "common/octahedral"
#import "common/lighting"
#import "common/fullscreen"
#import "common/ray_guides"
#import "common/reflection_trace"

// The guides DLSS ray reconstruction reads beside the frame, at the frame's size: each surface's specular albedo, the
// environment BRDF over its reflectance as combine weighs its reflection; its world normal, its roughness in the fourth
// channel; how far its reflection ray went before it met what it reflects, and its indirect light ray where that is
// traced; and how far it is reconstructed from this frame rather than its history, `RAY_GUIDE_BIAS_WATER` over a
// puddle's water.

// The G-buffer, the wet surface, the lighting, and the reflections' trace: what each traced pixel met, the ray's length
// in alpha, at the size they are traced at, below none where untraced.
#import "generated/frame/ray_guides"

struct RayGuides {
  @location(0) specular_albedo: vec4<f32>,
  @location(1) normals: vec4<f32>,
  @location(2) hit_distance: f32,
  @location(3) bias: f32,
  @location(4) diffuse_hit_distance: f32,
};

@fragment
fn fs_ray_guides(in: FullscreenVarying) -> RayGuides {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return RayGuides(vec4<f32>(0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0), RAY_GUIDE_FAR_HIT, 0.0, RAY_GUIDE_FAR_HIT);
  }

  let stored: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let albedo: vec4<f32> = vec4<f32>(authored_colour(stored.rgb), stored.a);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let wet: vec4<f32> = load_frame_target(wet_surface, texel);
  let facing: f32 = saturate(dot(normal, -normalize(position)));
  let roughness: f32 = traced_roughness(albedo.a, wet.a + surface_variance(normal_target, depth_target, texel, wet.g));
  let head_on: vec3<f32> = reflectance_head_on(albedo.rgb, albedo.a, wet.r, material.z);
  let intensity: f32 = select(1.0, lighting.reflections.z, lighting.reflections.x > 0.5);
  let specular: vec3<f32> = environment_brdf(head_on, roughness, facing) * intensity;
  let world_normal: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(normal, 0.0)).xyz);
  // The trace's texel over the pixel, at the size it was traced at.
  let size: vec2<i32> = vec2<i32>(textureDimensions(traced));
  let at: vec2<i32> = min(vec2<i32>(in.clip.xy * vec2<f32>(size) / camera.viewport.xy), size - 1);
  let met: f32 = textureLoad(traced, at, 0).a;
  // The indirect light's texel over the pixel, traced at half size: none where its ray met nothing.
  let bounced: f32 = textureLoad(diffuse_hits, min(texel / 2, vec2<i32>(textureDimensions(diffuse_hits)) - 1), 0).r;

  return RayGuides(vec4<f32>(specular, 1.0), vec4<f32>(world_normal, roughness), select(RAY_GUIDE_FAR_HIT, met, met >= 0.0),
    RAY_GUIDE_BIAS_WATER * saturate(wet.g), select(RAY_GUIDE_FAR_HIT, bounced, bounced > 0.0));
}
