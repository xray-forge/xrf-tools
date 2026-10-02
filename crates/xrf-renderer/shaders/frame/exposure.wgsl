#import "common/lighting"

// `phase_luminance`: the frame combine finished measured in 64 by 64 cells, then the scale every later tonemap
// multiplies by adapted towards the middle gray, read by the next frame as the engine's `s_tonemap` is.

struct ExposureState {
  adapted: f32,
  pad0: f32,
  pad1: f32,
  pad2: f32,
  cells: array<f32, 4096>,
};

// `MiddleGray`: the scale is `target / (luminance * weight + floor)`, moved towards by `blend` of the way.
struct ExposureParams {
  target_gray: f32,
  weight: f32,
  floor_luminance: f32,
  blend: f32,
};

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> state: ExposureState;
@group(0) @binding(2) var<uniform> params: ExposureParams;

// `LUMINANCE_VECTOR` (`common_defines.h`).
const LUMINANCE: vec3<f32> = vec3<f32>(0.3, 0.38, 0.22);

const CELLS: u32 = 64u;
const CELL_SAMPLES: u32 = 4u;
const REDUCE: u32 = 256u;

var<workgroup> partial: array<f32, 256>;

// `bloom_luminance_1`: each cell's luminance, the scaled colour's times two, as the bloom target it reads is twice
// the average it was built from.
@compute @workgroup_size(64)
fn measure(@builtin(global_invocation_id) id: vec3<u32>) {
  let index: u32 = id.x;

  if (index >= CELLS * CELLS) {
    return;
  }

  let size: vec2<f32> = vec2<f32>(textureDimensions(scene));
  let cell: vec2<f32> = vec2<f32>(f32(index % CELLS), f32(index / CELLS));
  var total: f32 = 0.0;

  for (var sample: u32 = 0u; sample < CELL_SAMPLES * CELL_SAMPLES; sample++) {
    let offset: vec2<f32> = (vec2<f32>(f32(sample % CELL_SAMPLES), f32(sample / CELL_SAMPLES)) + 0.5)
      / f32(CELL_SAMPLES);
    let at: vec2<i32> = vec2<i32>((cell + offset) / f32(CELLS) * size);

    total += dot(untonemap(saturate(textureLoad(scene, at, 0).rgb)), LUMINANCE) * 2.0;
  }

  state.cells[index] = total / f32(CELL_SAMPLES * CELL_SAMPLES);
}

// `bloom_luminance_3`: the cells averaged, and the adapted scale moved towards what brings them to the middle gray.
// Held between a 128th and twenty, as the engine's own clamp meant to.
@compute @workgroup_size(256)
fn adapt(@builtin(local_invocation_index) index: u32) {
  var total: f32 = 0.0;

  for (var cell: u32 = index; cell < CELLS * CELLS; cell += REDUCE) {
    total += state.cells[cell];
  }

  partial[index] = total;
  workgroupBarrier();

  for (var stride: u32 = REDUCE / 2u; stride > 0u; stride /= 2u) {
    if (index < stride) {
      partial[index] += partial[index + stride];
    }

    workgroupBarrier();
  }

  if (index == 0u) {
    let luminance: f32 = partial[0] / f32(CELLS * CELLS);
    let scale: f32 = params.target_gray / (luminance * params.weight + params.floor_luminance);

    state.adapted = clamp(mix(state.adapted, scale, params.blend), 1.0 / 128.0, 20.0);
  }
}
