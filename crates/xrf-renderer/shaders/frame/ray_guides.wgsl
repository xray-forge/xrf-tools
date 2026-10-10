#import "common/camera"
#import "common/octahedral"
#import "common/lighting"
#import "common/fullscreen"
#import "common/reflection_trace"

// The guides DLSS ray reconstruction reads beside the frame, at the frame's size: each surface's specular albedo, the
// environment BRDF over its reflectance as combine weighs its reflection; its world normal, its roughness in the fourth
// channel; and how far its reflection ray went before it met what it reflects.

// The G-buffer, the wet surface, the lighting, and the reflections' trace: what each traced pixel met, the ray's length
// in alpha, at the size they are traced at, below none where untraced.
#import "generated/frame/ray_guides"

// Metres a reflection stands away where no ray was traced or met anything: the sky's, the prefiltered surroundings'.
const FAR_HIT: f32 = 1000.0;

struct RayGuides {
  @location(0) specular_albedo: vec4<f32>,
  @location(1) normals: vec4<f32>,
  @location(2) hit_distance: f32,
};

@fragment
fn fs_ray_guides(in: FullscreenVarying) -> RayGuides {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return RayGuides(vec4<f32>(0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0), FAR_HIT);
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let wet: vec4<f32> = textureLoad(wet_surface, texel, 0);
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

  return RayGuides(vec4<f32>(specular, 1.0), vec4<f32>(world_normal, roughness), select(FAR_HIT, met, met >= 0.0));
}
