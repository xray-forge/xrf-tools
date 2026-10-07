enable wgpu_binding_array;

#import "grass/records"
#import "generated/grass/colours"

// Each detail model's colour where its tufts stand, which a traced hit on grassed ground blends towards: its base
// texture's solid texels (alpha over its reference) averaged at the texture's own resolution, a workgroup a model. A
// read of a coarse mip took the transparent texels' dark colour into the average and greyed the grass towards black.

@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var texture_sampler: sampler;

// Reads across and down the texture, a lane a column.
const COLOUR_READS: u32 = 64u;

var<workgroup> sums: array<vec4<f32>, 64>;

@compute @workgroup_size(64)
fn cs_grass_colours(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) lane: u32) {
  let model: GrassModel = grass_models[group.x];
  var sum: vec4<f32> = vec4<f32>(0.0);

  for (var row: u32 = 0u; row < COLOUR_READS; row++) {
    let at: vec2<f32> = (vec2<f32>(f32(lane), f32(row)) + 0.5) / f32(COLOUR_READS);
    let texel: vec4<f32> = textureSampleLevel(textures[model.texture], texture_sampler, at, 0.0);
    let solid: f32 = select(0.0, 1.0, texel.a > model.alpha_reference);

    sum += vec4<f32>(texel.rgb * solid, solid);
  }

  sums[lane] = sum;
  workgroupBarrier();

  for (var stride: u32 = COLOUR_READS / 2u; stride > 0u; stride /= 2u) {
    if (lane < stride) {
      sums[lane] += sums[lane + stride];
    }

    workgroupBarrier();
  }

  if (lane == 0u) {
    let total: vec4<f32> = sums[0];

    grass_colours[group.x] = vec4<f32>(total.rgb / max(total.a, 1.0), total.a / f32(COLOUR_READS * COLOUR_READS));
  }
}
