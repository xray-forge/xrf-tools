#import "common/puddle_site"
#import "grass/planting_bindings"

// The share of a puddle's radius within which no tuft stands: short of its soft rim, where the ground is only damp.
const PUDDLE_BARE: f32 = 0.85;

// Whether a tuft's foot `world`, in renderer space as the wetting reads its sites, stands in a puddle's water this
// frame: the puddles in its cell and those about it, each the stretched, turned blob the wetting draws about its site
// (`site_puddle` in `frame/wet_patch.wgsl`, without the noise that moves its border), within `PUDDLE_BARE` of its
// radius.
fn is_in_puddle(world: vec3<f32>) -> bool {
  if (puddle_window.state.x < 0.5) {
    return false;
  }

  let cell: f32 = puddle_window.shape.z;
  let cells: f32 = floor(puddle_window.surface.z * 2.0 / cell);
  let corner: vec2<f32> = puddle_window.surface.xy - puddle_window.surface.z;
  let origin: vec2<f32> = site_origin(corner, cell);
  let at: vec2<f32> = floor(world.xz / cell) - origin;

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

      if (length(stretched) < site.w * PUDDLE_BARE && abs(world.y - site.z) < band) {
        return true;
      }
    }
  }

  return false;
}
