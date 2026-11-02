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
// traced; how far it is reconstructed from this frame rather than its history, `RAY_GUIDE_BIAS_WATER` over a puddle's
// water; and its diffuse albedo as combine lights with it, linear as the guides must be, none over the sky.

// The G-buffer, the wet surface, the lighting, and the reflections' trace: what each traced pixel met, the ray's length
// in alpha, at the size they are traced at, below none where untraced.
#import "generated/frame/ray_guides"

struct RayGuides {
  @location(0) specular_albedo: vec4<f32>,
  @location(1) normals: vec4<f32>,
  @location(2) hit_distance: f32,
  @location(3) bias: f32,
  @location(4) diffuse_hit_distance: f32,
  @location(5) diffuse_albedo: vec4<f32>,
};

// The texel of a target traced at a size of its own over the frame's point `clip`.
fn texel_over(clip: vec2<f32>, size: vec2<u32>) -> vec2<i32> {
  return min(vec2<i32>(clip * vec2<f32>(size) / camera.viewport.xy), vec2<i32>(size) - 1);
}

@fragment
fn fs_ray_guides(in: FullscreenVarying) -> RayGuides {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return RayGuides(vec4<f32>(0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0), RAY_GUIDE_FAR_HIT, 0.0, RAY_GUIDE_FAR_HIT,
      vec4<f32>(0.0));
  }

  let stored: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let albedo: vec4<f32> = vec4<f32>(authored_colour(stored.rgb), stored.a);
  let material: vec4<f32> = textureLoad(material_target, texel, 0);
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let wet: vec4<f32> = load_frame_target(wet_surface, texel);
  let facing: f32 = saturate(dot(normal, -normalize(position)));
  let surface: SurfaceMaterial = surface_material(albedo, material.z,
    water_film(wet, load_frame_target(wet_reach, texel)),
    wet.a + surface_variance(normal_target, depth_target, texel, wet.g), is_light_anchored(lighting));
  let roughness: f32 = surface.roughness;
  let head_on: vec3<f32> = surface.head_on;
  let intensity: f32 = select(1.0, lighting.reflections.z, lighting.reflections.x > 0.5);
  let specular: vec3<f32> = environment_brdf(head_on, roughness, facing) * surface.shadowing * intensity;
  let world_normal: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(normal, 0.0)).xyz);
  // How far the reflection's ray and the indirect light's went, each at the size it was traced at: none where the
  // indirect light's met nothing.
  let met: f32 = textureLoad(traced, texel_over(in.clip.xy, textureDimensions(traced)), 0).a;
  let bounced: f32 = textureLoad(diffuse_hits, texel_over(in.clip.xy, textureDimensions(diffuse_hits)), 0).r;

  return RayGuides(vec4<f32>(specular, 1.0), vec4<f32>(world_normal, roughness), select(RAY_GUIDE_FAR_HIT, met, met >= 0.0),
    RAY_GUIDE_BIAS_WATER * saturate(wet.g), select(RAY_GUIDE_FAR_HIT, bounced, bounced > 0.0), vec4<f32>(albedo.rgb, 1.0));
}
