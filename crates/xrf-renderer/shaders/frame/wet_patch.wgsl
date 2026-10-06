#import "common/camera"
#import "common/octahedral"
#import "common/rain_cover"
#import "common/fullscreen"
#import "common/wet"
#import "common/puddle_site"

// `rain_patch_normal` (`r3_rendertarget_draw_rain.cpp`): where the rain reaches a surface near the camera, splashes on
// what faces up and water running down what stands, as a normal bent in view space, and how wet it is in alpha, weighed
// by the albedo's brightness. Computed in the engine's axes, so the ripples run as the game's do.

// The G-buffer and its material marks, the rain's cover, `s_water` (a volume of rippling normals as its slices) and
// `s_waterFall` (the normals of water running down), the enhanced wetting's rain ripples, puddle noise and puddle
// normals, and the settings.
#import "generated/frame/wet_patch"

// The engines' values: metres past which the rain wets nothing, how fast the splashes' volume runs, how far the water
// tilts the normal up, how strongly and how fast the flow bends and runs.
fn engine_value(vanilla: f32, extended: f32) -> f32 {
  return select(vanilla, extended, wet.is_extended > 0.5);
}

// How much of the rain reaches a point facing `world_normal`, from none to one, by the cover as `cover_reached` reads
// it, the wetness creeping in under a cover by the level's soak; `pixel` turns the creep's taps.
fn rain_reached(world: vec3<f32>, world_normal: vec3<f32>, pixel: vec2<f32>) -> f32 {
  let noise: f32 = cover_noise(pixel + wet.cover_state.w);

  return cover_reached(cover, cover_pyramid, wet.cover, wet.cover_state.xy, world, world_normal, wet.cover_state.z,
    noise);
}

/// `GetNVNMap`: the splashes' volume at a place and a moment, repeating every way and blended between its slices, its
// normal in `w`, `y` and `z`, laid flat.
fn splash_normal(place: vec2<f32>, moment: f32) -> vec3<f32> {
  let layers: f32 = f32(textureNumLayers(splash));
  let along: f32 = fract(moment) * layers - 0.5;
  let first: f32 = floor(along);
  let near: i32 = i32((first + layers) % layers);
  let far: i32 = i32((first + 1.0) % layers);
  let water: vec4<f32> = mix(
    textureSampleLevel(splash, wet_sampler, place, near, 0.0),
    textureSampleLevel(splash, wet_sampler, place, far, 0.0),
    along - first,
  ) - 0.5;

  return vec3<f32>(water.w * 6.0, engine_value(0.0, 0.1), water.z * 6.0);
}

// `GetWaterNMap`: the flow's normal at a point of it, laid flat.
fn flow_normal(at: vec2<f32>) -> vec3<f32> {
  let water: vec3<f32> = (textureSampleLevel(flow, wet_sampler, at, 0.0).xzy - 0.5) * 2.0 * engine_value(0.3, 0.4);

  return vec3<f32>(water.x, engine_value(0.0, 0.1), water.z);
}

// The engine's wetting of a surface (`rain_patch_normal`): how its normal is bent by the splashes on what faces up and
// the water running down what stands near the camera, in view space, by the falling rain's density, and how wet it is,
// by the level's wetness, which settles at that density under a steady rain and dries after it, from the G-buffer's
// normal and point in view space, the world they stand in and how much of the rain reaches it.
struct EngineWetting {
  bent: vec3<f32>,
  wetness: f32,
};

fn engine_wetting(
  normal: vec3<f32>,
  position: vec3<f32>,
  world: vec3<f32>,
  world_normal: vec3<f32>,
  reached: f32,
) -> EngineWetting {
  // The engine's axes: `z` negated.
  let place: vec3<f32> = vec3<f32>(world.x, world.y, -world.z);
  let facing: vec3<f32> = vec3<f32>(world_normal.x, world_normal.y, -world_normal.z);
  let fade: f32 = 1.0 - smoothstep(5.0, engine_value(20.0, 25.0), -position.z);
  // `-dot(Ldynamic_dir, N)`, the rain falling straight down.
  let up: f32 = facing.y;
  let near: f32 = reached * fade * fade * saturate(up * 10.0 + 10.0 * 0.5 + 0.5);
  let wetness: f32 = near * wet.puddles.x;
  let upward: f32 = max(up, 0.0);
  let splashed: vec3<f32> = splash_normal(place.xz, wet.time * engine_value(3.0, 1.0));
  let along: vec3<f32> = place / 2.0;
  let slide: f32 = ceil((1.0 - upward) * 10.0) * 0.1 * engine_value(0.5, 0.3);
  let fall_x: vec3<f32> = flow_normal(vec2<f32>(along.z, along.y + wet.time * slide));
  let fall_z: vec3<f32> = flow_normal(vec2<f32>(along.x, along.y + wet.time * slide));
  // Nothing on the weapon in hand, which the viewer has none of.
  let applied: f32 = near * wet.density * smoothstep(0.8, 0.9, length(position));
  let water: vec3<f32> = splashed * (upward * applied) + fall_x.yxz * (abs(facing.x) * applied) +
    fall_z.zxy * (abs(facing.z) * applied);

  return EngineWetting((camera.view * vec4<f32>(water.x, water.y, -water.z, 0.0)).xyz, wetness);
}

// The share of the reflections' reach (`wet.reach.y`, metres) over which the wetness fades out at its end.
const REACH_FADE: f32 = 0.8;
// The gloss a wet surface is brought up to at most where the reflections trace: the engine's gain of up to 0.8 over
// a bright surface's own gloss made a mirror of it, and the sky sparkled in every bump of its normal map as foil;
// below a puddle's water, so wet ground never reads as water.
const WET_GLOSS_MOST: f32 = 0.6;

// How wet a surface reads, from none to one: the engine's, by its splashes' fade over 5 to 25 m, or where the
// reflections trace (`wet.reach.x`) as far as the rain reaches, so they have wet gloss to trace over the whole ground
// and not a ring about the camera, as much as the rain `reached` it (a tunnel's vault, a bridge, a roof or a tree's
// crown keeps what is under it dry), fading out to the reach.
fn surface_wetness(wetting: EngineWetting, reached: f32, world_normal: vec3<f32>, position: vec3<f32>) -> f32 {
  if (wet.reach.x < 0.5) {
    return wetting.wetness;
  }

  let facing_up: f32 = saturate(world_normal.y * 10.0 + 10.0 * 0.5 + 0.5);

  return reached * wet.puddles.x * facing_up * smoothstep(wet.reach.y, wet.reach.y * REACH_FADE, length(position));
}

// The gloss the wetness gains over the surface's own, from how wet it reads and the engine's gain a unit of wetness
// brings (0.8 in Anomaly, 1 in Extended): the engine's where it alone wets, no more than `WET_GLOSS_MOST` where the
// reflections trace.
fn wet_gloss_gain(soaked: f32, gloss: f32) -> f32 {
  let gain: f32 = soaked * select(0.8, 1.0, wet.is_extended > 0.5);

  return select(gain, min(gain, max(WET_GLOSS_MOST - gloss, 0.0)), wet.reach.x > 0.5);
}

@fragment
fn fs_wet_patch(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Reversed: nought where nothing was drawn.
  if (depth <= 0.0) {
    discard;
  }

  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let normal: vec3<f32> = normalize(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let world: vec3<f32> = camera_unproject(ndc, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_normal: vec3<f32> = normalize(rotation * normal);
  let reached: f32 = rain_reached(world, world_normal, in.clip.xy);
  let wetting: EngineWetting = engine_wetting(normal, position, world, world_normal, reached);
  let wetness: f32 = surface_wetness(wetting, reached, world_normal, position);

  // The bent normal packed as the enhanced patch packs it; how wet the engine reads the surface, by the albedo's
  // brightness, which the apply pass darkens by; and the gloss that gains, the engine's own factor folded in.
  let soaked: f32 = wetness * dot(albedo.rgb, vec3<f32>(0.33));

  return vec4<f32>(octahedral_encode(normalize(normal + wetting.bent)), wet_gloss_gain(soaked, albedo.a), soaked);
}

// The puddles' rain rings: how fast each of their three layers runs, where the second and third are moved to, and the
// metres within which they ripple, fading out over them.
const RIPPLE_REACH: f32 = 15.0;
const RIPPLE_LAYER_SPEEDS: vec3<f32> = vec3<f32>(1.05, 1.31, 1.58);
const RIPPLE_LAYER_OFFSETS: vec4<f32> = vec4<f32>(0.5, 0.25, 0.31, 0.5);
// The share of the puddles' distance over which they fade out.
const PUDDLE_FADE: f32 = 0.15;
// The reflectivity setting at which a puddle's water reflects as water does, the default; a lower one reflects less,
// a higher one more.
const WATER_REFLECTIVITY: f32 = 0.4;
// Metres a puddle's noise repeats over, and its ripples' normals; and how strongly rain ripples a puddle.
const PUDDLE_TILE: f32 = 60.0;
const PUDDLE_RIPPLE_TILE: f32 = 12.0;
const PUDDLE_RIPPLE_SHARE: f32 = 0.5;
// A puddle's shape about its site: as long as wide at most by `PUDDLE_STRETCH` (`common/puddle_site`), its border
// moved in and out by a noise every `PUDDLE_EDGE_TILE` metres and by the puddles' own over `PUDDLE_TILE`, a share of
// its radius; and the share of its radius its border softens over.
const PUDDLE_EDGE_TILE: f32 = 8.0;
const PUDDLE_EDGE_SHAPE: f32 = 0.35;
const PUDDLE_NOISE_SHAPE: f32 = 0.25;
const PUDDLE_SOFTNESS: f32 = 0.12;
// Metres a puddle's water stands over the ground at its site's spot, its plane lying along the ground's own up there;
// the most water the tint sees, a puddle's depth, whatever the plane's height over a dip in the ground says; how far
// past its rim the ground around it stays damp with the water it soaked, a share of its radius, and how wet, as the
// engine's wetness reads.
const PUDDLE_LEVEL: f32 = 0.03;
const PUDDLE_DEEPEST: f32 = 0.12;
const PUDDLE_HALO: f32 = 0.4;
const PUDDLE_HALO_WET: f32 = 0.25;
// How much the coat's coverage varies with the puddles' own noise, for floating film and debris.
const PUDDLE_FILM: f32 = 0.15;
// How the level's puddles differ from stretch to stretch, smoothly over tens of metres so a joined puddle never shows a
// seam: their water's roughness, from a still mirror's to a scummed one's; how murky, scaling the depth the floor is
// seen through; and their waves' phase, in tiles, and strength. Metres the variation repeats over, its three readings
// turned and offset off one another.
const WATER_ROUGHNESS: vec2<f32> = vec2<f32>(0.02, 0.06);
const PUDDLE_MURK: vec2<f32> = vec2<f32>(0.6, 1.3);
const WAVE_PHASE: f32 = 0.5;
const WAVE_STRENGTH: vec2<f32> = vec2<f32>(0.6, 1.4);
const VARIATION_TILES: vec3<f32> = vec3<f32>(37.0, 53.0, 43.0);
// Gusts: patches of stronger wind drifting downwind across the level (`wet.gust`), two readings of the regional noise
// at metres apart, the second drifting half as fast; the calm waves under a gust's lull and its strongest, and the rain
// rings' share of it, so a gust ruffles some puddles while others lie still.
const GUST_TILES: vec2<f32> = vec2<f32>(70.0, 29.0);
const GUST_WAVES: vec2<f32> = vec2<f32>(0.3, 1.9);
const GUST_RINGS: vec2<f32> = vec2<f32>(0.8, 1.2);
// A puddle's rim: the share of its radius from which its water roughens to a wet film's, the roughness it reaches at
// its edge, and how much of its clear coat the rim gives up, so a puddle's reflection fades in from a damp edge.
const RIM_START: f32 = 0.55;
const RIM_ROUGHNESS: f32 = 0.12;
const RIM_COAT: f32 = 0.5;
// How far the ripples bend a puddle's normal at most, as a slope: at full strength the water looked like foil.
const RIPPLE_BEND_MOST: f32 = 0.08;
// The lean built into the water's wave map by its generator, which the water shader expects and a puddle takes off:
// its calm water lies level on average.
const WAVE_TILT: vec2<f32> = vec2<f32>(-0.105, -0.104);
// The least a filtered normal's length is divided by for the variance it lost.
const TOKSVIG_LEAST: f32 = 0.05;

// Metres a point may stand off its site's height, beside how far the placing let the ground rise and fall across it
// (`SITE_LEVEL`), and still be in its water; and how far under the level surface's highest lowest about it it may stand
// and still be the topmost surface: a stone or a post in a puddle stays dry, as does what lies under anything standing
// over the ground.
const PUDDLE_DEPTH: f32 = 0.15;
const PUDDLE_TOP: f32 = 0.25;
// The least a surface faces up to hold a puddle.
const PUDDLE_UP: f32 = 0.85;

// One layer of rain rings: each texel a ring arriving on its own clock (arrival, direction x, direction y, clock),
// spreading and fading; `setup` is its speed, strength and how many rings a ripple spreads. The tilt in `xy`, and in
// `z` the tilt's variance the map's filtering folded away (Toksvig): over a footprint spanning many rings their
// directions average shorter, and what the tilt lost in length the surface keeps as roughness.
fn ripple_layer(texel: vec4<f32>, setup: vec3<f32>) -> vec3<f32> {
  let direction: vec2<f32> = texel.yz * 2.0 - 1.0;
  let moment: f32 = fract(texel.w + wet.time * setup.x);
  let age: f32 = moment - 1.0 + texel.x;
  let rings: f32 = clamp(age * setup.z, 0.0, 4.0);
  let height: f32 = saturate(0.7 - moment) * texel.x * sin(rings * 3.141592) * smoothstep(4.0, 0.0, rings) * setup.y;
  let kept: f32 = length(direction);

  return vec3<f32>(direction * height, height * height * max(1.0 - kept, 0.0) / max(kept, TOKSVIG_LEAST));
}

// How a place on the ground changes to the next pixel across and down, which the maps are filtered by.
struct PlaceGradients {
  across: vec2<f32>,
  down: vec2<f32>,
};

// Rain rings at a place on a puddle, three layers over each other, fading out to `RIPPLE_REACH` metres away: the tilt
// in `xy`, its lost variance in `z`; `gradients` are the place's.
fn rain_ripples(place: vec2<f32>, gradients: PlaceGradients, setup: vec3<f32>, distance: f32) -> vec3<f32> {
  let near: f32 = RIPPLE_REACH - distance;

  if (near < 0.0) {
    return vec3<f32>(0.0);
  }

  let first: vec4<f32> = textureSampleGrad(ripples, wet_sampler, place, gradients.across, gradients.down);
  let second: vec4<f32> = textureSampleGrad(ripples, wet_sampler, place * 0.61 + RIPPLE_LAYER_OFFSETS.xy,
    gradients.across * 0.61, gradients.down * 0.61);
  let third: vec4<f32> = textureSampleGrad(ripples, wet_sampler, place * 0.87 + RIPPLE_LAYER_OFFSETS.zw,
    gradients.across * 0.87, gradients.down * 0.87);
  let summed: vec3<f32> = ripple_layer(first, vec3<f32>(RIPPLE_LAYER_SPEEDS.x * setup.x, setup.yz)) +
    ripple_layer(second, vec3<f32>(RIPPLE_LAYER_SPEEDS.y * setup.x, setup.yz)) +
    ripple_layer(third, vec3<f32>(RIPPLE_LAYER_SPEEDS.z * setup.x, setup.yz));
  let fade: f32 = near / RIPPLE_REACH;

  return vec3<f32>(clamp(summed.xy * fade, vec2<f32>(-1.0), vec2<f32>(1.0)), summed.z * fade * fade);
}

// Metres either side the ground's slope under a puddle is read at: across a puddle, past its bumps; and the texels
// either side of its own the lowest map's lowest is taken over (`LOWEST_REACH` in `frame/puddle_sites.wgsl`).
const GROUND_SPAN: f32 = 1.5;
const LOWEST_REACH: f32 = 1.0;

// The level surface's lowest height at a place in the world, none outside its map.
fn lowest_at(place: vec2<f32>) -> f32 {
  let uv: vec2<f32> = clamp((place - wet.surface.xy) / (wet.surface.z * 2.0) + 0.5, vec2<f32>(0.0), vec2<f32>(0.999));

  return textureLoad(surface_lowest, vec2<i32>(uv * wet.surface_shape.x), 0).x;
}

// The ground's own up under a point in the world, from the level surface's lowest heights `GROUND_SPAN` metres either
// way: its slope across a puddle, past its bumps, which a puddle's water lies along so that the ground rising beyond
// it runs alongside its reflection rather than across it.
fn ground_up(world: vec3<f32>) -> vec3<f32> {
  let across: f32 = lowest_at(world.xz - vec2<f32>(GROUND_SPAN, 0.0)) - lowest_at(world.xz + vec2<f32>(GROUND_SPAN, 0.0));
  let down: f32 = lowest_at(world.xz - vec2<f32>(0.0, GROUND_SPAN)) - lowest_at(world.xz + vec2<f32>(0.0, GROUND_SPAN));

  return normalize(vec3<f32>(across, 2.0 * GROUND_SPAN, down));
}

// Whether a point is the topmost surface of the level there, by the level surface's lowest heights about it.
fn is_topmost(world: vec3<f32>) -> bool {
  let uv: vec2<f32> = (world.xz - wet.surface.xy) / (wet.surface.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return false;
  }

  let at: vec2<i32> = vec2<i32>(uv * wet.surface_shape.x);
  let last: vec2<i32> = vec2<i32>(i32(wet.surface_shape.x) - 1);
  var highest: f32 = -1e9;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<i32> = vec2<i32>(i32(corner & 1u), i32(corner >> 1u));

    highest = max(highest, textureLoad(surface_lowest, clamp(at + offset, vec2<i32>(0), last), 0).x);
  }

  return world.y > highest - PUDDLE_TOP;
}

// What a point has of the puddles about it: how much of one it is, how deep its water stands there in metres, how
// much of the damp halo around one, the water's up, the plane's, and how far out towards its rim it stands, a share of
// its radius.
struct SitePuddle {
  inside: f32,
  depth: f32,
  halo: f32,
  up: vec3<f32>,
  rim: f32,
};

// How much of a puddle a point is: this frame's puddles in its cell and those about it, each a whole stretched blob
// about its site as big as the placing made it, turned and stretched by the site's hash, a soft rim, seated within
// the band the placing let the ground rise and fall over. Its water is one plane a puddle, `PUDDLE_LEVEL` over the
// ground at the site's spot and lying along the ground's own up there (the site's height is the lowest map's, which
// lies under the ground on a slope by its window's half width times the slope, taken back), so nothing of it steps
// with the map's texels; how far that plane stands over the ground is the depth the water's tint sees, at most a
// puddle's, and it never cuts the puddle, since the placing already found the disc level. `border` moves each blob's
// edge in and out, a share of its radius; `footprint` is the metres a pixel spans there, over which a border is
// softened at least, so a far one does not shimmer. Nothing outside the level surface's map.
fn site_puddle(world: vec3<f32>, border: f32, footprint: f32) -> SitePuddle {
  let cell: f32 = wet.surface_shape.z;
  let cells: f32 = floor(wet.surface.z * 2.0 / cell);
  let corner: vec2<f32> = wet.surface.xy - wet.surface.z;
  let origin: vec2<f32> = site_origin(corner, cell);
  // The point's cell of the sites' map: its world cell past the map's first.
  let at: vec2<f32> = floor(world.xz / cell) - origin;
  let texel: f32 = wet.surface.z * 2.0 / wet.surface_shape.x;
  var puddle: SitePuddle = SitePuddle(0.0, 0.0, 0.0, vec3<f32>(0.0, 1.0, 0.0), 1.0);

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
      let distance: f32 = length(stretched) / site.w + border;
      let softness: f32 = max(PUDDLE_SOFTNESS, 1.5 * footprint / site.w);
      let band: f32 = PUDDLE_DEPTH + SITE_LEVEL.x + SITE_LEVEL.y * site.w;
      let seated: f32 = 1.0 - smoothstep(band * 0.75, band, abs(world.y - site.z));
      let inside: f32 = (1.0 - smoothstep(1.0 - softness, 1.0, distance)) * seated;
      let halo: f32 = (1.0 - smoothstep(1.0, 1.0 + PUDDLE_HALO, distance)) * seated;

      puddle.halo = max(puddle.halo, halo);

      if (inside > puddle.inside) {
        // The water's plane: along the ground's own up at the site's spot, through the ground there.
        let up: vec3<f32> = ground_up(vec3<f32>(site.x, site.z, site.y));
        let under: f32 = (LOWEST_REACH + 0.5) * texel * (abs(up.x) + abs(up.z)) / up.y;
        let lift: f32 = site.z + under + PUDDLE_LEVEL * wet.puddle_state.y - dot(up.xz, offset) / up.y - world.y;

        puddle.inside = inside;
        puddle.depth = clamp(lift, 0.0, PUDDLE_DEEPEST * wet.puddle_state.y);
        puddle.up = up;
        puddle.rim = distance;
      }
    }
  }

  return puddle;
}

// How the puddles differ at a place in the engine's axes: in `x` their water's roughness, in `y` their murk, in `z`
// their waves' phase and strength, each from nothing to one.
fn puddle_variation(place: vec2<f32>) -> vec3<f32> {
  return vec3<f32>(
    textureSampleLevel(puddle_noise, wet_sampler, place / VARIATION_TILES.x + vec2<f32>(0.13, 0.71), 0.0).r,
    textureSampleLevel(puddle_noise, wet_sampler, place.yx / VARIATION_TILES.y + vec2<f32>(0.59, 0.27), 0.0).r,
    textureSampleLevel(puddle_noise, wet_sampler, vec2<f32>(place.x, -place.y) / VARIATION_TILES.z + 0.43, 0.0).r,
  );
}

// How gusty the wind is over a place in renderer space, from a lull to a gust's strongest.
fn gustiness(place: vec2<f32>) -> f32 {
  let drift: vec2<f32> = wet.gust.xy * wet.time;
  let broad: f32 = textureSampleLevel(puddle_noise, wet_sampler, (place - drift) / GUST_TILES.x + 0.21, 0.0).r;
  let fine: f32 =
    textureSampleLevel(puddle_noise, wet_sampler, (place.yx - drift.yx * 0.5) / GUST_TILES.y + 0.67, 0.0).r;

  return smoothstep(0.35, 0.75, broad * 0.7 + fine * 0.3);
}

// The gloss a puddle's water of a roughness has, as `reflection_roughness` reads gloss back.
fn water_gloss(roughness: f32) -> f32 {
  let glassy: f32 = 1.0 - sqrt(roughness);

  return glassy * glassy;
}

// What the enhanced wetting writes: into the light target, as the engine's wetting does, the normal packed into `xy`
// (bent as the engine bends it, a puddle levelling it to the ground's own up and its rings bending that), what the
// wetness and a puddle's water add to the gloss into `z`, and how wet the engine reads the surface into `w`, which
// darkens it as the engine's wetting does; into the wet surface, what the puddles add over it: how much of the surface
// a puddle's clear coat of water covers (the puddle scaled by the reflectivity setting against water's, and by
// floating film), how much a puddle, the water's depth over the ground, metres, which a reflection's ray leaves from
// and the water darkens by, and the variance of the rings' tilt the maps' filtering folded away, which the
// reflection's roughness takes up.
struct WetPatchTargets {
  @location(0) patched: vec4<f32>,
  @location(1) surface: vec4<f32>,
};

// The puddles: the engine's own wetting of every surface, as the engine's patch writes it, and this frame's puddles on
// level, open terrain over it, each a layer of water lying along the ground's own up, rippling. The reflections
// traced over the wet gloss are the rest of the wet look; nothing else is added to a surface.
@fragment
fn fs_wet_patch_enhanced(in: FullscreenVarying) -> WetPatchTargets {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    discard;
  }

  let is_terrain: bool = has_mark(textureLoad(material_target, texel, 0).a, MARK_TERRAIN);
  let albedo: vec4<f32> = textureLoad(albedo_target, texel, 0);
  let gloss: f32 = albedo.a;
  var normal: vec3<f32> = normalize(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let world: vec3<f32> = camera_unproject(ndc, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_normal: vec3<f32> = normalize(rotation * normal);
  // The engine's axes: `z` negated.
  let place: vec3<f32> = vec3<f32>(world.x, world.y, -world.z);
  let distance: f32 = -position.z;
  let gradients: PlaceGradients = PlaceGradients(dpdx(place.xz), dpdy(place.xz));
  let footprint: f32 = max(length(gradients.across), length(gradients.down));
  let reached: f32 = rain_reached(world, world_normal, in.clip.xy);
  let wetting: EngineWetting = engine_wetting(normal, position, world, world_normal, reached);
  // How wet the engine reads the surface, by the albedo's brightness, as far as the reflections want it.
  var wetness: f32 = surface_wetness(wetting, reached, world_normal, position) * dot(albedo.rgb, vec3<f32>(0.33));
  var water: vec3<f32> = wetting.bent;
  var variance: f32 = 0.0;

  // Puddles: whole puddles where the placing put them this frame, faded out over the last of the puddle distance,
  // each a layer of water over the ground, rippled by the rain.
  var puddle: f32 = 0.0;
  var floating: f32 = 1.0;
  var lift: f32 = 0.0;
  var water_roughness: f32 = WATER_ROUGHNESS.x;

  // On the ground alone, terrain facing up and the topmost surface there: never a roof, a deck or a ceiling.
  let is_ground: bool = is_terrain && world_normal.y > PUDDLE_UP && is_topmost(world);

  if (is_ground && wet.puddle_state.y > 0.0 && length(position) < wet.puddle_state.x) {
    let noise: f32 = textureSampleGrad(puddle_noise, wet_sampler, place.xz / PUDDLE_TILE,
      gradients.across / PUDDLE_TILE, gradients.down / PUDDLE_TILE).r;
    let edge: f32 = textureSampleGrad(puddle_noise, wet_sampler, place.zx / PUDDLE_EDGE_TILE + 0.37,
      gradients.across.yx / PUDDLE_EDGE_TILE, gradients.down.yx / PUDDLE_EDGE_TILE).r;
    let border: f32 = (edge - 0.5) * PUDDLE_EDGE_SHAPE + (noise - 0.5) * PUDDLE_NOISE_SHAPE;
    let fade: f32 = smoothstep(wet.puddle_state.x, wet.puddle_state.x * (1.0 - PUDDLE_FADE), length(position));
    // The cover only reaches some tens of metres; a site is on open ground already, so a puddle does not wait for it.
    let found: SitePuddle = site_puddle(world, border, footprint);

    let variation: vec3<f32> = puddle_variation(place.xz);
    let rim: f32 = smoothstep(RIM_START, 1.0, found.rim);

    puddle = found.inside * fade;
    lift = min(found.depth * mix(PUDDLE_MURK.x, PUDDLE_MURK.y, variation.y), PUDDLE_DEEPEST * wet.puddle_state.y);
    floating = (1.0 - PUDDLE_FILM * noise) * (1.0 - RIM_COAT * rim);
    water_roughness = mix(mix(WATER_ROUGHNESS.x, WATER_ROUGHNESS.y, variation.x), RIM_ROUGHNESS, rim);
    wetness = max(wetness, PUDDLE_HALO_WET * found.halo * fade * (1.0 - found.inside));

    if (puddle > 0.0) {
      let pace: f32 = wet.time * (0.01 + saturate(wet.density * 1.5) * 0.008);
      let uv: vec2<f32> = place.xz / PUDDLE_RIPPLE_TILE;
      let waves: vec2<f32> = uv + (variation.z - 0.5) * WAVE_PHASE;
      let across: vec2<f32> = gradients.across / PUDDLE_RIPPLE_TILE;
      let down: vec2<f32> = gradients.down / PUDDLE_RIPPLE_TILE;
      // The water's calm waves, two scrolls of the map read filtered, its lean taken off; what the filtering left of
      // their normals' length is the tilt they lost, kept as variance.
      let first: vec3<f32> = textureSampleGrad(puddle_normal, wet_sampler, waves + vec2<f32>(0.0, pace), across, down)
        .xyz * 2.0 - 1.0 - vec3<f32>(WAVE_TILT, 0.0);
      let second: vec3<f32> = textureSampleGrad(puddle_normal, wet_sampler, waves - vec2<f32>(0.33, pace), across, down)
        .xyz * 2.0 - 1.0 - vec3<f32>(WAVE_TILT, 0.0);
      let kept_length: f32 = 0.5 * (length(first) + length(second));
      let gust: f32 = gustiness(world.xz);
      let calm_scale: f32 = saturate(0.05 + saturate(wet.density * 1.5) * 0.1) * 0.666 *
        mix(WAVE_STRENGTH.x, WAVE_STRENGTH.y, variation.z) * mix(GUST_WAVES.x, GUST_WAVES.y, gust);
      let drop_scale: f32 = wet.density * 3.0 * wet.puddles.w * PUDDLE_RIPPLE_SHARE * 0.15 *
        mix(GUST_RINGS.x, GUST_RINGS.y, gust);
      let drops: vec3<f32> = rain_ripples(uv * 1.3, PlaceGradients(across * 1.3, down * 1.3), vec3<f32>(0.85, 1.0, 10.0),
        distance);
      let tilt: vec2<f32> = 0.5 * (first.xy + second.xy) * calm_scale + drops.xy * drop_scale;
      // Bent no further than the most; the variance is the whole tilt's, since what the clamp takes off is roughness too.
      let bend: vec2<f32> = tilt * min(1.0, RIPPLE_BEND_MOST / max(length(tilt), 1e-4)) * puddle;
      let spread: f32 = calm_scale * calm_scale * max(1.0 - kept_length, 0.0) / max(kept_length, TOKSVIG_LEAST) +
        drop_scale * drop_scale * drops.z;
      let bent: vec3<f32> = (camera.view * vec4<f32>(bend.x, 0.0, -bend.y, 0.0)).xyz;

      normal = normalize(mix(normal, (camera.view * vec4<f32>(found.up, 0.0)).xyz, puddle));
      water = water * (1.0 - puddle) + bent;
      variance = spread * puddle * puddle;
    }
  }

  // What the gloss gains: the engine's wet gloss over its own, capped below a mirror, a puddle's water's.
  let glossed: f32 = mix(min(wetness * select(0.8, 1.0, wet.is_extended > 0.5), max(WET_GLOSS_MOST - gloss, 0.0)),
    water_gloss(water_roughness) - gloss, puddle);

  return WetPatchTargets(
    vec4<f32>(octahedral_encode(normalize(normal + water)), glossed, wetness),
    vec4<f32>(puddle * floating * wet.puddles.z / WATER_REFLECTIVITY, puddle, lift, variance),
  );
}
