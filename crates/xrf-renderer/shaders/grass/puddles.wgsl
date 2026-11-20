#import "common/puddle_site"
#import "grass/planting_bindings"

// The share of its size a tuft stands at by the puddles this frame, its foot `world`, in renderer space as the
// wetting reads its sites, `from_eye` metres from the eye: the puddles in its cell and those about it, each the
// stretched, turned blob the wetting draws about its site (`site_puddle` in `frame/wet_patch.wgsl`, without the noise
// that moves its border), as far as the wetting draws it (`puddle_drawn`), the least any of them leaves
// (`puddle_tuft`).
fn puddle_kept(world: vec3<f32>, from_eye: f32) -> f32 {
  let drawn: f32 = puddle_drawn(from_eye, puddle_window.state.y);

  if (puddle_window.state.x < 0.5 || drawn <= 0.0) {
    return 1.0;
  }

  let cell: f32 = puddle_window.shape.z;
  let cells: f32 = floor(puddle_window.surface.z * 2.0 / cell);
  let corner: vec2<f32> = puddle_window.surface.xy - puddle_window.surface.z;
  let origin: vec2<f32> = site_origin(corner, cell);
  let at: vec2<f32> = floor(world.xz / cell) - origin;
  var kept: f32 = 1.0;

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let near: vec2<f32> = at + vec2<f32>(f32(x), f32(y));

      if (any(near < vec2<f32>(0.0)) || any(near >= vec2<f32>(cells))) {
        continue;
      }

      let site: vec4<f32> = textureLoad(puddles, vec2<i32>(near), 0);

      if (site.w <= 0.0 || length(world.xz - site.xy) > site.w * 2.0) {
        continue;
      }

      let hash: vec3<f32> = site_hash(origin + near);
      let turn: f32 = hash.y * 6.2831853;
      let across: vec2<f32> = vec2<f32>(cos(turn), sin(turn));
      let offset: vec2<f32> = world.xz - site.xy;
      let stretched: vec2<f32> = vec2<f32>(dot(offset, across), dot(offset, vec2<f32>(-across.y, across.x))) /
        vec2<f32>(1.0, mix(1.0, 1.0 / PUDDLE_STRETCH, hash.z));
      let band: f32 = SITE_LEVEL.x + SITE_LEVEL.y * site.w + 0.15;

      if (abs(world.y - site.z) < band) {
        kept = min(kept, puddle_tuft(length(stretched) / site.w, drawn));
      }
    }
  }

  return kept;
}
