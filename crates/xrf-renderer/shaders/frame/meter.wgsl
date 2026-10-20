#import "common/camera"
#import "common/scene_light"
#import "common/luminance"
#import "common/srgb"

// A spot meter at the view's centre: the scene's light averaged over a square of texels, taken back from what the
// scene holds at the frame's exposure, how many stops it was exposed above middle grey, and what the G-buffer holds of
// the surfaces under it, averaged over the texels any surface covers.

#import "generated/frame/meter"

// Texels the spot reaches out from its centre: a square of nine a side.
const SPOT_REACH: i32 = 4;

@compute @workgroup_size(1)
fn cs_meter() {
  let size: vec2<i32> = vec2<i32>(textureDimensions(scene));
  let centre: vec2<i32> = size / 2;
  let scale: f32 = frame_scale(lighting, exposure);
  var light: vec3<f32> = vec3<f32>(0.0);
  var drawn: f32 = 0.0;
  var albedo: f32 = 0.0;
  var gloss: f32 = 0.0;
  var diffuse: f32 = 0.0;
  var specular: f32 = 0.0;
  var distance: f32 = 0.0;

  for (var y: i32 = -SPOT_REACH; y <= SPOT_REACH; y++) {
    for (var x: i32 = -SPOT_REACH; x <= SPOT_REACH; x++) {
      let texel: vec2<i32> = clamp(centre + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let depth: f32 = textureLoad(depth_target, texel, 0);

      light += scene_light(textureLoad(scene, texel, 0).rgb, scale);

      if (depth > 0.0) {
        let surface: vec4<f32> = textureLoad(albedo_target, texel, 0);
        let gathered: vec4<f32> = textureLoad(light_target, texel, 0);

        drawn += 1.0;
        albedo += luminance(srgb_decoded(surface.rgb));
        gloss += surface.a;
        diffuse += luminance(gathered.rgb);
        specular += gathered.a;
        distance += -camera_view_position(vec2<f32>(texel) + 0.5, depth).z;
      }
    }
  }

  let texels: f32 = f32((2 * SPOT_REACH + 1) * (2 * SPOT_REACH + 1));
  let shown: f32 = luminance(light / texels);
  let covered: f32 = max(drawn, 1.0);

  sample.luminance = shown;
  sample.stops = log2(max(shown * scale, 1e-6) / MIDDLE_GREY);
  sample.exposure = scale;
  sample.drawn = drawn / texels;
  sample.albedo = albedo / covered;
  sample.gloss = gloss / covered;
  sample.diffuse = diffuse / covered;
  sample.specular = specular / covered;
  sample.distance = distance / covered;
}
