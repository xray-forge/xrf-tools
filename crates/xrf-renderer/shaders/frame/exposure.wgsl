#import "common/lighting"

// `phase_luminance`: the frame's high part measured in 64 by 64 cells, then the scale every later tonemap multiplies
// by adapted towards the middle gray, read by the next frame as the engine's `s_tonemap` is.

// The high part (`rt_Generic_1`), the scaled colour over `def_hdr` that the bloom is built from.
#import "generated/frame/exposure"

// `LUMINANCE_VECTOR` (`common_defines.h`).
const LUMINANCE: vec3<f32> = vec3<f32>(0.3, 0.38, 0.22);

const CELLS: u32 = 64u;
const CELL_SAMPLES: u32 = 4u;
const REDUCE: u32 = 256u;

var<workgroup> partial: array<f32, 256>;

// `bloom_luminance_1`: each cell's luminance as the bloom build it reads holds it, which `phase_luminance` measures
// before the blur: twice the high part, held to one by its eight bits, back over `def_hdr`.
@compute @workgroup_size(64)
fn measure(@builtin(global_invocation_id) id: vec3<u32>) {
  let index: u32 = id.x;

  if (index >= CELLS * CELLS) {
    return;
  }

  let size: vec2<f32> = vec2<f32>(textureDimensions(high));
  let cell: vec2<f32> = vec2<f32>(f32(index % CELLS), f32(index / CELLS));
  var total: f32 = 0.0;

  for (var sample: u32 = 0u; sample < CELL_SAMPLES * CELL_SAMPLES; sample++) {
    let offset: vec2<f32> = (vec2<f32>(f32(sample % CELL_SAMPLES), f32(sample / CELL_SAMPLES)) + 0.5)
      / f32(CELL_SAMPLES);
    let at: vec2<i32> = vec2<i32>((cell + offset) / f32(CELLS) * size);

    total += dot(min(textureLoad(high, at, 0).rgb * 2.0, vec3<f32>(1.0)), LUMINANCE) * DEF_HDR;
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
