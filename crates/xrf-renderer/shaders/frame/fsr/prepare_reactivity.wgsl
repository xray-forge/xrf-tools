#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr3upscaler_prepare_reactivity.h`, at the drawn size: the reactive, disocclusion, shading change and
// accumulation masks, the accumulation the next frame starts from, and the thin features to lock. FSR writes each lock
// to the display pixel its sample falls in; the accumulation gathers it from here instead. The transparency and
// composition mask is the plants (grass and leaves) the wind swayed this frame: the sway changes what an alpha-tested
// card covers, which their motion does not carry, so their locks are taken off and their history clamped as AMD
// advises for such surfaces. A plant standing still keeps its locks.

#import "common/marks"
@group(0) @binding(1) var dilated_depth: texture_2d<f32>;
@group(0) @binding(2) var dilated_motion: texture_2d<f32>;
// The reconstructed depth of the frame before, a depth's bits a drawn texel.
@group(0) @binding(3) var<storage, read> reconstructed: array<u32>;
@group(0) @binding(4) var reactive_mask: texture_2d<f32>;
@group(0) @binding(5) var shading_change: texture_2d<f32>;
@group(0) @binding(6) var luma: texture_2d<f32>;
// How much of the history each drawn texel trusted the frame before.
@group(0) @binding(7) var previous_accumulation: texture_2d<f32>;
@group(0) @binding(8) var linear_sampler: sampler;
// The G-buffer's material target, whose alpha holds the surface marks.
@group(0) @binding(9) var material_target: texture_2d<f32>;

// `Ksep`: the depth separation a pixel of the view can tell, per unit of distance.
const DEPTH_SEPARATION: f32 = 1.37e-5;
// A neighbour whose luma lies within this share of the neighbourhood's range of the centre's is similar to it.
const SIMILAR_THRESHOLD: f32 = 0.9;
// The weakest thin feature locked.
const LOCK_STRENGTH_MIN: f32 = 1.0 / 100.0;
// The transparency and composition value of a plant: one removes its lock outright.
const PLANT_COMPOSITION: f32 = 1.0;

struct PrepareReactivityOutput {
  @location(0) masks: vec4<f32>,
  @location(1) accumulation: vec4<f32>,
  @location(2) new_locks: vec4<f32>,
};

fn load_reconstructed(at: vec2<f32>) -> f32 {
  let texel: vec2<i32> = clamped_texel(at, fsr.render_size);

  return bitcast<f32>(reconstructed[u32(texel.y) * u32(fsr.render_size.x) + u32(texel.x)]);
}

// `ComputeDisocclusions`: how far the surface stands behind what stood where it was the frame before, against the
// separation the view's resolution can tell at its distance.
fn disocclusion(uv: vec2<f32>, motion: vec2<f32>, view_depth: f32) -> f32 {
  let moved: vec2<f32> = motion * f32(to_4k_velocity(motion) > still_threshold(min(view_depth, FSR_FP16_MAX)));
  let footprint: BilinearFootprint = bilinear_footprint(uv + moved, fsr.render_size);
  let half_viewport: f32 = length(fsr.render_size * 0.5);
  var sum: f32 = 0.0;
  var weight_sum: f32 = 0.0;
  var is_potential: bool = true;

  for (var corner: i32 = 0; corner < 4 && is_potential; corner++) {
    let weight: f32 = footprint.weights[corner];

    if (weight > RECONSTRUCTED_DEPTH_WEIGHT_THRESHOLD) {
      let previous: f32 = to_view_depth(load_reconstructed(footprint.base + footprint_corner(corner)));
      let difference: f32 = view_depth - previous;

      is_potential = difference > FSR_FP32_MIN;

      if (is_potential) {
        let required: f32 = DEPTH_SEPARATION * half_viewport * max(view_depth, previous);

        sum += saturate(required / difference) * weight;
        weight_sum += weight;
      }
    }
  }

  return select(0.0, saturate(1.0 - sum / weight_sum), is_potential && weight_sum > 0.0);
}

// `ComputeMotionDivergence`: how much slower what the motion lands on moves, the closer it stands and the faster this
// texel moves; a texel whose motion leaves the screen lands on nothing, as a load off a texture reads.
fn motion_divergence(uv: vec2<f32>, motion: vec2<f32>, depth: f32) -> f32 {
  let landing: vec2<f32> = trunc((uv + motion) * fsr.render_size);
  let is_landing_on_screen: bool = is_on_screen(landing, fsr.render_size);
  let texel: vec2<i32> = clamped_texel(landing, fsr.render_size);
  let landed_depth: f32 = select(0.0, textureLoad(dilated_depth, texel, 0).x, is_landing_on_screen);
  let landed_motion: vec2<f32> = select(vec2<f32>(0.0), textureLoad(dilated_motion, texel, 0).xy,
    is_landing_on_screen);
  let landed_velocity: f32 = to_4k_velocity(landed_motion);
  let velocity: f32 = to_4k_velocity(motion);
  let distance_factor: f32 = min_over_max(to_view_depth(landed_depth), to_view_depth(depth));
  // Still, it diverges from nothing: `saturate` of `0 / 0` is zero on the hardware FSR targets.
  let slower: f32 = select(0.0, saturate(landed_velocity / velocity), velocity > 0.0);

  return (1.0 - slower) * distance_factor * saturate(velocity / 10.0);
}

// `DilateReactiveMasks`: the most reactive of the nine.
fn dilated_reactive(position: vec2<f32>) -> f32 {
  var reactive: f32 = 0.0;

  for (var index: i32 = 0; index < 9; index++) {
    reactive = max(reactive, textureLoad(reactive_mask, clamped_texel(position + NEAREST_ORDER[index],
      fsr.render_size), 0).x);
  }

  return reactive;
}

// `ComputeShadingChange`, the jitter applied again as the shading change took it off.
fn shading_changed(uv: vec2<f32>) -> f32 {
  let shading_uv: vec2<f32> = clamped_uv(uv - fsr.jitter / fsr.render_size, fsr.shading_size);

  return saturate(textureSampleLevel(shading_change, linear_sampler, shading_uv, 0.0).x);
}

// `UpdateAccumulation`: how far the history may be trusted here, the frame before's trust where the surface was on the
// screen, lost to a change of shading and below none where it was disoccluded; and what the next frame starts from, a
// third of a frame more.
fn update_accumulation(uv: vec2<f32>, motion: vec2<f32>, disoccluded: f32, changed: f32) -> vec2<f32> {
  let reprojected: vec2<f32> = uv + motion;
  var accumulation: f32 = 0.0;

  if (is_uv_inside(reprojected)) {
    accumulation = saturate(textureSampleLevel(previous_accumulation, linear_sampler,
      clamped_uv(reprojected, fsr.render_size), 0.0).x);
  }

  accumulation = mix(accumulation, 0.0, changed);
  accumulation = mix(accumulation, min(MIN_DISOCCLUSION_ACCUMULATION, accumulation), disoccluded);
  accumulation *= f32(round(accumulation * 100.0) > 1.0);

  return vec2<f32>(accumulation, saturate(accumulation + ACCUMULATION_ADDED_PER_FRAME));
}

// `ComputeThinFeatureConfidence`: how strongly a texel whose luma stands above or below all it differs from around it,
// and that no quadrant of similar texels surrounds, is a thin feature to lock.
fn thin_feature(position: vec2<f32>) -> f32 {
  // The nine in AMD's order: the centre, then row by row, `1 2 3 / 4 0 5 / 6 7 8`.
  let offsets: array<vec2<f32>, 9> = array<vec2<f32>, 9>(
    vec2<f32>(0.0, 0.0), vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 0.0),
    vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 1.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
  );
  var samples: array<f32, 9>;
  var least: f32 = FSR_FP32_MAX;
  var most: f32 = FSR_FP32_MIN;

  for (var index: i32 = 0; index < 9; index++) {
    samples[index] = textureLoad(luma, clamped_texel(position + offsets[index], fsr.render_size), 0).x;
    least = min(least, samples[index]);
    most = max(most, samples[index]);
  }

  let nucleus: f32 = samples[0];
  let range: f32 = most - least;
  var pattern: u32 = 1u;
  var dissimilar_min: f32 = FSR_FP32_MAX;
  var dissimilar_max: f32 = 0.0;

  for (var index: i32 = 1; index < 9; index++) {
    // `|s - n| / range < 0.9` without the division, which a flat neighbourhood would leave to NaN: no neighbour of one
    // is similar.
    if (range > 0.0 && abs(samples[index] - nucleus) < SIMILAR_THRESHOLD * range) {
      pattern |= 1u << u32(index);
    } else {
      dissimilar_min = min(dissimilar_min, samples[index]);
      dissimilar_max = max(dissimilar_max, samples[index]);
    }
  }

  if (!(nucleus > dissimilar_max || nucleus < dissimilar_min)) {
    return 0.0;
  }

  // The quadrants, upper left, upper right, lower left and lower right, each with the centre.
  let quadrants: vec4<u32> = vec4<u32>(0x17u, 0x2du, 0xd1u, 0x1a1u);

  for (var index: i32 = 0; index < 4; index++) {
    if ((pattern & quadrants[index]) == quadrants[index]) {
      return 0.0;
    }
  }

  return 1.0 - least / most;
}

// `DilateTransparencyAndCompositionMasks`: the mask where the drawn texel is a plant the wind moved.
fn transparency_and_composition(texel: vec2<i32>) -> f32 {
  let marks: f32 = textureLoad(material_target, texel, 0).a;

  return select(0.0, PLANT_COMPOSITION, has_mark(marks, MARK_PLANT) && has_mark(marks, MARK_MOVED));
}

@fragment
fn fs_prepare_reactivity(in: FullscreenVarying) -> PrepareReactivityOutput {
  let position: vec2<f32> = floor(in.clip.xy);
  let texel: vec2<i32> = clamped_texel(position, fsr.render_size);
  let uv: vec2<f32> = (position + 0.5) / fsr.render_size;
  let motion: vec2<f32> = textureLoad(dilated_motion, texel, 0).xy;
  let depth: f32 = textureLoad(dilated_depth, texel, 0).x;
  let disoccluded: f32 = disocclusion(uv, motion, to_view_depth(depth));
  let changed: f32 = max(dilated_reactive(position), shading_changed(uv));
  let accumulation: vec2<f32> = update_accumulation(uv, motion, disoccluded, changed);
  let lock: f32 = thin_feature(position);
  var out: PrepareReactivityOutput;

  out.masks = vec4<f32>(max(motion_divergence(uv, motion, depth), transparency_and_composition(texel)), disoccluded,
    changed, accumulation.x);
  out.accumulation = vec4<f32>(accumulation.y, 0.0, 0.0, 1.0);
  out.new_locks = vec4<f32>(select(0.0, lock, lock > LOCK_STRENGTH_MIN), 0.0, 0.0, 1.0);

  return out;
}
