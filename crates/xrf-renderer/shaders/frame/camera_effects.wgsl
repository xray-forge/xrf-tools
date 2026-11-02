#import "common/fullscreen"
#import "common/noise"
#import "common/lens"
#import "generated/frame/camera_effects"

// The camera the resolved frame is shown through, each stage a triangle over its target. After the resolve, so the
// temporal resolve and the upscalers never see a blurred edge whose motion no longer matches it; the drawn depth and
// motion are read only by the inputs stage, over the drawn texels under each shown pixel, the nearest depth and the
// longest motion among them.
//
// The depth of field is a thin lens's circle of confusion gathered at half the frame's size over tiles of the nearest
// and farthest circles, the near tiles spread over those they reach (Jimenez, SIGGRAPH 2014; Abadie, SIGGRAPH 2018):
// a far sample covers a pixel by no more than the pixel's own circle, so the background never bleeds over what stands
// in focus, while the near field spreads over both sides of its silhouette. The motion blur is a reconstruction filter
// along each tile's longest motion, weighing each sample by which of the two stands in front (McGuire et al., I3D 2012;
// Guertin et al., HPG 2014). The lens dims its corners and splits its colours (`common/lens`).

// The half-size texels a side of a depth of field tile, sixteen shown pixels (`CameraEffectsUniform::FOCUS_TILE`).
const FOCUS_TILE_TEXELS: i32 = 8;
const FOCUS_TILE_PIXELS: f32 = 16.0;
// The shown pixels a side of a block of the motion's first reduction (`CameraEffectsUniform::MOTION_BLOCK`).
const MOTION_BLOCK: i32 = 8;
// A gather's samples over its circle, on a golden-angle spiral.
const FOCUS_SAMPLES: u32 = 48u;
const GOLDEN_ANGLE: f32 = 2.39996323;
// How strongly a near sample's coverage of the gather's circle lays the near field over what lies behind it.
const NEAR_COVERAGE: f32 = 2.0;
// Samples along a pixel's motion.
const MOTION_SAMPLES: u32 = 12u;
// Shown pixels of motion below which a tile is left as it stands.
const MOTION_LEAST: f32 = 0.5;
// How far apart two depths stand, as a share of the nearer and in metres, before one is wholly in front.
const DEPTH_SOFTNESS: f32 = 0.02;
const DEPTH_SOFTNESS_LEAST: f32 = 0.05;

// Metres from the camera a stored depth stands at, by the projection's depth terms.
fn distance_of(stored: f32) -> f32 {
  let terms: vec4<f32> = camera_effects.depth;

  // Parallel rays store `z` scaled and offset; a perspective stores it divided by `-z`.
  if (terms.w > 0.0) {
    return (terms.y - stored) / terms.x;
  }

  return terms.y / max(terms.x - stored * terms.z, 1e-9);
}

// The circle of confusion's radius in shown pixels a point at `distance` metres blurs to: positive behind the focus,
// negative before it, held to the widest.
fn confusion_of(distance: f32) -> f32 {
  let lens: vec4<f32> = camera_effects.lens;
  let z: f32 = max(distance, lens.y * 1.001);
  let diameter: f32 = lens.x * lens.y * (z - lens.z) / (z * (lens.z - lens.y));
  let widest: f32 = camera_effects.focus.x;

  return clamp(diameter * 0.5 * lens.w, -widest, widest);
}

// A texel of an image held to its edges.
fn clamped(image: texture_2d<f32>, texel: vec2<i32>) -> vec4<f32> {
  return textureLoad(image, clamp(texel, vec2<i32>(0), vec2<i32>(textureDimensions(image)) - 1), 0);
}

// The longer of two motions.
fn longer(first: vec2<f32>, second: vec2<f32>) -> vec2<f32> {
  return select(first, second, dot(second, second) > dot(first, first));
}

// The metres and the motion under a shown pixel: of the drawn texels about it, the nearest depth and the longest
// motion, the motion in shown pixels over half the shutter, held to the longest blur.
@fragment
fn fs_inputs(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let drawn: vec2<f32> = in.clip.xy * camera_effects.shown.zw * camera_effects.drawn.xy;
  let centre: vec2<i32> = vec2<i32>(floor(drawn));
  let last: vec2<i32> = vec2<i32>(camera_effects.drawn.xy) - 1;
  var nearest: f32 = 0.0;
  var motion: vec2<f32> = vec2<f32>(0.0);

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let texel: vec2<i32> = clamp(centre + vec2<i32>(x, y), vec2<i32>(0), last);

      // Reversed depth: the nearest stores the most.
      nearest = max(nearest, textureLoad(depth_target, texel, 0));
      motion = longer(motion, textureLoad(motion_target, texel, 0).xy);
    }
  }

  let swept: vec2<f32> = motion * camera_effects.shown.xy * camera_effects.motion.x * 0.5;
  let length_of: f32 = length(swept);
  let held: vec2<f32> = swept * min(1.0, camera_effects.motion.y / max(length_of, 1e-6));

  return vec4<f32>(distance_of(nearest), held, 0.0);
}

// The longest motion over a block of the shown frame.
@fragment
fn fs_motion_reduce(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let corner: vec2<i32> = vec2<i32>(in.clip.xy) * MOTION_BLOCK;
  let size: vec2<i32> = vec2<i32>(textureDimensions(inputs));
  var motion: vec2<f32> = vec2<f32>(0.0);

  for (var y: i32 = 0; y < MOTION_BLOCK; y++) {
    for (var x: i32 = 0; x < MOTION_BLOCK; x++) {
      let texel: vec2<i32> = corner + vec2<i32>(x, y);

      if (all(texel < size)) {
        motion = longer(motion, textureLoad(inputs, texel, 0).yz);
      }
    }
  }

  return vec4<f32>(motion, 0.0, 0.0);
}

// The longest motion over a tile, from its blocks.
@fragment
fn fs_motion_tiles(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let blocks: i32 = i32(camera_effects.motion.z) / MOTION_BLOCK;
  let corner: vec2<i32> = vec2<i32>(in.clip.xy) * blocks;
  let size: vec2<i32> = vec2<i32>(textureDimensions(source));
  var motion: vec2<f32> = vec2<f32>(0.0);

  for (var y: i32 = 0; y < blocks; y++) {
    for (var x: i32 = 0; x < blocks; x++) {
      let texel: vec2<i32> = corner + vec2<i32>(x, y);

      if (all(texel < size)) {
        motion = longer(motion, textureLoad(source, texel, 0).xy);
      }
    }
  }

  return vec4<f32>(motion, 0.0, 0.0);
}

// The longest motion over a tile and its eight neighbours, which is the longest that can reach into it.
@fragment
fn fs_motion_neighbours(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let tile: vec2<i32> = vec2<i32>(in.clip.xy);
  var motion: vec2<f32> = vec2<f32>(0.0);

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      motion = longer(motion, clamped(source, tile + vec2<i32>(x, y)).xy);
    }
  }

  return vec4<f32>(motion, 0.0, 0.0);
}

// How much of `nearer` stands in front of `farther`, softly over a span growing with distance.
fn in_front(nearer: f32, farther: f32) -> f32 {
  let softness: f32 = max(DEPTH_SOFTNESS * min(nearer, farther), DEPTH_SOFTNESS_LEAST);

  return saturate(1.0 - (nearer - farther) / softness);
}

// How much of a sample `offset` pixels away a blur of `radius` covers: falling off along it, or whole within it.
fn cone(offset: f32, radius: f32) -> f32 {
  return saturate(1.0 - offset / max(radius, 1e-3));
}

fn cylinder(offset: f32, radius: f32) -> f32 {
  return 1.0 - smoothstep(0.95 * radius, 1.05 * radius, offset);
}

// The frame blurred along its tile's longest motion, its tile read a little off the pixel by noise so the tiles'
// edges do not show, each sample weighed by whether it blurs over the pixel or the pixel over it.
@fragment
fn fs_motion_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = in.clip.xy;
  let texel: vec2<i32> = vec2<i32>(pixel);
  let colour: vec4<f32> = textureLoad(source, texel, 0);
  let noise: vec2<f32> = gradient_noise_2(pixel);
  let tile: f32 = camera_effects.motion.z;
  let tile_texel: vec2<i32> = vec2<i32>(floor((pixel + (noise - 0.5) * tile * 0.5) / tile));
  let widest: vec2<f32> = clamped(tiles, tile_texel).xy;
  let widest_length: f32 = length(widest);

  if (widest_length < MOTION_LEAST) {
    return colour;
  }

  let centre: vec4<f32> = textureLoad(inputs, texel, 0);
  let centre_length: f32 = max(length(centre.yz), MOTION_LEAST);
  var weight: f32 = 1.0 / centre_length;
  var sum: vec3<f32> = colour.rgb * weight;

  for (var index: u32 = 0u; index < MOTION_SAMPLES; index++) {
    let along: f32 = mix(-1.0, 1.0, (f32(index) + noise.x) / f32(MOTION_SAMPLES));
    let offset: f32 = abs(along) * widest_length;
    let sample_texel: vec2<i32> = vec2<i32>(floor(pixel + widest * along));
    let sample_inputs: vec4<f32> = clamped(inputs, sample_texel);
    let sample_length: f32 = length(sample_inputs.yz);
    let front: f32 = in_front(centre.x, sample_inputs.x);
    let back: f32 = in_front(sample_inputs.x, centre.x);
    let covered: f32 = back * cone(offset, sample_length) + front * cone(offset, centre_length)
      + 2.0 * cylinder(offset, sample_length) * cylinder(offset, centre_length);

    weight += covered;
    sum += clamped(source, sample_texel).rgb * covered;
  }

  return vec4<f32>(sum / weight, colour.a);
}

// The frame at half its size, each texel the four shown pixels under it weighed down by their brightness so a lone
// bright pixel does not flare into a disc, with the circle of the nearest of them.
@fragment
fn fs_focus_reduce(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let corner: vec2<i32> = vec2<i32>(in.clip.xy) * 2;
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;
  var nearest: f32 = 3.4e38;

  for (var y: i32 = 0; y < 2; y++) {
    for (var x: i32 = 0; x < 2; x++) {
      let texel: vec2<i32> = corner + vec2<i32>(x, y);
      let colour: vec3<f32> = clamped(source, texel).rgb;
      let share: f32 = 1.0 / (1.0 + max(colour.r, max(colour.g, colour.b)));

      sum += colour * share;
      weight += share;
      nearest = min(nearest, clamped(inputs, texel).x);
    }
  }

  return vec4<f32>(sum / weight, confusion_of(nearest));
}

// The widest near and far circles over a tile of the half-size frame.
@fragment
fn fs_focus_tiles(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let corner: vec2<i32> = vec2<i32>(in.clip.xy) * FOCUS_TILE_TEXELS;
  let size: vec2<i32> = vec2<i32>(textureDimensions(source));
  var near: f32 = 0.0;
  var far: f32 = 0.0;

  for (var y: i32 = 0; y < FOCUS_TILE_TEXELS; y++) {
    for (var x: i32 = 0; x < FOCUS_TILE_TEXELS; x++) {
      let texel: vec2<i32> = corner + vec2<i32>(x, y);

      if (all(texel < size)) {
        let confusion: f32 = textureLoad(source, texel, 0).a;

        near = max(near, -confusion);
        far = max(far, confusion);
      }
    }
  }

  return vec4<f32>(near, far, 0.0, 0.0);
}

// A tile's near circle widened to the widest near circle of the tiles about it that reaches it; its far circle its
// own, as the background spreads over nothing in front of it.
@fragment
fn fs_focus_dilate(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let tile: vec2<i32> = vec2<i32>(in.clip.xy);
  let reach: i32 = i32(camera_effects.focus.z);
  let own: vec4<f32> = clamped(source, tile);
  var near: f32 = own.x;

  for (var y: i32 = -reach; y <= reach; y++) {
    for (var x: i32 = -reach; x <= reach; x++) {
      let neighbour: f32 = clamped(source, tile + vec2<i32>(x, y)).x;
      let between: f32 = f32(max(max(abs(x), abs(y)) - 1, 0)) * FOCUS_TILE_PIXELS;

      if (neighbour > between) {
        near = max(near, neighbour);
      }
    }
  }

  return vec4<f32>(near, own.y, 0.0, 0.0);
}

// The half-size frame gathered over the widest circle reaching each texel, near and far apart: a far sample covers
// the texel by no more than the texel's own circle, a near one by its own; the near field is laid over the far by
// how much of the circle it covers, which the combine lays over the sharp frame.
@fragment
fn fs_focus_gather(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = in.clip.xy;
  let texel: vec2<i32> = vec2<i32>(pixel);
  let centre: vec4<f32> = textureLoad(source, texel, 0);
  let circles: vec4<f32> = clamped(tiles, texel / FOCUS_TILE_TEXELS);
  // Radii in half-size texels from here on.
  let own: f32 = max(centre.a, 0.0) * 0.5;
  let radius: f32 = max(circles.x * 0.5, abs(centre.a) * 0.5);

  if (radius < 0.5) {
    return vec4<f32>(centre.rgb, 0.0);
  }

  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let turn: f32 = gradient_noise(pixel) * 6.2831853;
  var far_sum: vec3<f32> = vec3<f32>(0.0);
  var far_weight: f32 = 0.0;
  var near_sum: vec3<f32> = vec3<f32>(0.0);
  var near_weight: f32 = 0.0;

  for (var index: u32 = 0u; index < FOCUS_SAMPLES; index++) {
    let distance: f32 = sqrt((f32(index) + 0.5) / f32(FOCUS_SAMPLES)) * radius;
    let angle: f32 = f32(index) * GOLDEN_ANGLE + turn;
    let at: vec2<f32> = (pixel + vec2<f32>(cos(angle), sin(angle)) * distance) / size;
    let sampled: vec4<f32> = textureSampleLevel(source, source_sampler, at, 0.0);
    let confusion: f32 = sampled.a * 0.5;
    let far: f32 = saturate(min(max(confusion, 0.0), own) - distance + 1.0);
    let near: f32 = saturate(-confusion - distance + 1.0);

    far_sum += sampled.rgb * far;
    far_weight += far;
    near_sum += sampled.rgb * near;
    near_weight += near;
  }

  let far_colour: vec3<f32> = select(centre.rgb, far_sum / far_weight, far_weight > 0.0);
  let near_colour: vec3<f32> = select(centre.rgb, near_sum / near_weight, near_weight > 0.0);
  let coverage: f32 = saturate(near_weight / f32(FOCUS_SAMPLES) * NEAR_COVERAGE);

  return vec4<f32>(mix(far_colour, near_colour, coverage), coverage);
}

// The gathered frame laid over the sharp one by the pixel's own circle, or by how much of a near field covers it.
@fragment
fn fs_focus_combine(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let sharp: vec4<f32> = textureLoad(source, texel, 0);
  let blurred: vec4<f32> = textureSampleLevel(beside, source_sampler, in.clip.xy * camera_effects.shown.zw, 0.0);
  let confusion: f32 = confusion_of(textureLoad(inputs, texel, 0).x);
  let share: f32 = max(smoothstep(0.5, 2.0, abs(confusion)), blurred.a);

  return vec4<f32>(mix(sharp.rgb, blurred.rgb, share), sharp.a);
}

// The lens's colour split towards the edges and its falloff towards the corners, on the linear frame.
@fragment
fn fs_lens(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy * camera_effects.shown.zw;
  let optics: vec4<f32> = camera_effects.optics;
  var colour: vec4<f32> = textureLoad(source, vec2<i32>(in.clip.xy), 0);

  if (optics.w > 0.0) {
    colour = vec4<f32>(lens_aberrated(source, source_sampler, uv, optics.w), colour.a);
  }

  return vec4<f32>(colour.rgb * lens_vignette(uv, optics.xy, optics.z), colour.a);
}
