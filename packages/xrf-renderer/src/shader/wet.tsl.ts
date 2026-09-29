import {
  abs,
  cameraViewMatrix,
  ceil,
  Discard,
  dot,
  float,
  Fn,
  getViewPosition,
  If,
  length,
  max,
  normalize,
  saturate,
  screenUV,
  select,
  smoothstep,
  sqrt,
  texture,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { IEngineValue } from "#/shader/engine-value";
import { isExtendedEngine, toEngineValue } from "#/shader/engine-value.tsl";
import { decodeOctahedral, encodeOctahedral } from "#/shader/octahedral-normal.tsl";
import { toRainCoverHeight } from "#/shader/rain.tsl";
import { IWetGlossInput } from "#/shader/wet-gloss-input";
import { IWetPatchInput } from "#/shader/wet-patch-input";
import { RAIN_COVER_RESOLUTION } from "#/visibility/rain-cover";

/** Metres a point may stand under its cover's texel and still take the rain, as a slope spans one. */
const COVER_BIAS: number = 0.2;

/** Metres from the view past which the rain wets nothing, faded out from five. */
const WET_REACH: IEngineValue = { extended: 25, vanilla: 20 };

/** How fast the splashes' volume runs through its slices, a slice a second at one. */
const SPLASH_RATE: IEngineValue = { extended: 1, vanilla: 3 };

/** How far the splashes and the flow tilt the normal up. */
const WATER_LIFT: IEngineValue = { extended: 0.1, vanilla: 0 };

/** How strongly the flow down a wall bends its normal. */
const FLOW_STRENGTH: IEngineValue = { extended: 0.4, vanilla: 0.3 };

/** How fast the water runs down, for each tenth a wall stands up. */
const FLOW_SPEED: IEngineValue = { extended: 0.3, vanilla: 0.5 };

/** How much of the wetness is added to the gloss. */
const WET_GLOSS: IEngineValue = { extended: 1, vanilla: 0.8 };

/**
 * Whether the rain reaches a point: four taps of the cover about it, as `shadow_rain` jitters four, each open where
 * the point stands at or above the first thing over its column.
 */
function toRainOpen(world: Node<"vec3">, input: IWetPatchInput): Node<"float"> {
  const texel: Node<"float"> = input.rain.window.z.mul(2 / RAIN_COVER_RESOLUTION);
  let open: Node<"float"> = float(0);

  for (const [x, z] of [
    [-0.5, -0.5],
    [0.5, -0.5],
    [-0.5, 0.5],
    [0.5, 0.5],
  ]) {
    const column: Node<"vec3"> = world.add(vec3(texel.mul(x), 0, texel.mul(z)));

    open = open.add(select(world.y.add(COVER_BIAS).greaterThanEqual(toRainCoverHeight(column, input.rain)), 1, 0));
  }

  return open.mul(0.25);
}

/** `GetNVNMap`: the splashes' volume at a place and a moment, its normal in `w`, `y` and `z`, laid flat. */
function toSplash(input: IWetPatchInput, place: Node<"vec2">, moment: Node<"float">): Node<"vec3"> {
  const water: Node<"vec4"> = input.wet.splash.sample(vec3(place, moment)).sub(0.5);

  return vec3(water.w.mul(6), toEngineValue(input.engine, WATER_LIFT), water.z.mul(6));
}

/** `GetWaterNMap`: the flow's normal at a point of it, laid flat. */
function toFlow(input: IWetPatchInput, at: Node<"vec2">): Node<"vec3"> {
  const { wet, engine } = input;
  const water: Node<"vec3"> = wet.flow.sample(at).xzy.sub(0.5).mul(2).mul(toEngineValue(engine, FLOW_STRENGTH));

  return vec3(water.x, toEngineValue(engine, WATER_LIFT), water.z);
}

/**
 * `rain_patch_normal`: where the rain reaches a surface near the camera, splashes on what faces up and water running
 * down what stands, as a normal bent in view space, and how wet it is in alpha, weighed by the albedo's brightness.
 * Computed in the engine's axes, so the ripples run as the game's do.
 *
 * @param input - The G-buffer and what wets it.
 * @returns The fragment: the patched view space normal, and the wetness.
 */
export function toWetPatchFragment(input: IWetPatchInput): Node<"vec4"> {
  return Fn(() => {
    const { textures, camera, wet, engine } = input;
    const depth: Node<"float"> = texture(textures.depth, screenUV).x;

    // Reversed: nought where nothing was drawn.
    If(depth.lessThanEqual(0), () => {
      Discard();
    });

    const albedo: Node<"vec3"> = texture(textures.albedo, screenUV).xyz;
    const normal: Node<"vec3"> = normalize(decodeOctahedral(texture(textures.normal, screenUV).xy));
    const position: Node<"vec3"> = getViewPosition(screenUV, depth, camera.projectionInverse);
    const world: Node<"vec3"> = camera.viewToWorld.mul(vec4(position, 1)).xyz;
    const worldNormal: Node<"vec3"> = normalize(camera.viewToWorld.mul(vec4(normal, 0)).xyz);
    // The engine's axes: `z` negated.
    const place: Node<"vec3"> = vec3(world.x, world.y, world.z.negate());
    const facing: Node<"vec3"> = vec3(worldNormal.x, worldNormal.y, worldNormal.z.negate());
    const far: Node<"float"> = toEngineValue(engine, WET_REACH);
    const fade: Node<"float"> = smoothstep(float(5), far, position.z.negate()).oneMinus();
    // `-dot(Ldynamic_dir, N)`, the rain falling straight down.
    const up: Node<"float"> = facing.y;
    const wetness: Node<"float"> = toRainOpen(world, input)
      .mul(fade.mul(fade))
      .mul(wet.density)
      .mul(saturate(up.mul(10).add(10 * 0.5 + 0.5)));
    const upward: Node<"float"> = max(up, 0);
    const splash: Node<"vec3"> = toSplash(input, place.xz, wet.time.mul(toEngineValue(engine, SPLASH_RATE)));
    const along: Node<"vec3"> = place.div(2);
    const slide: Node<"float"> = ceil(upward.oneMinus().mul(10)).mul(0.1).mul(toEngineValue(engine, FLOW_SPEED));
    const fallX: Node<"vec3"> = toFlow(input, vec2(along.z, along.y.add(wet.time.mul(slide))));
    const fallZ: Node<"vec3"> = toFlow(input, vec2(along.x, along.y.add(wet.time.mul(slide))));
    // Nothing on the weapon in hand, which the viewer has none of.
    const applied: Node<"float"> = wetness.mul(smoothstep(float(0.8), float(0.9), length(position)));
    const water: Node<"vec3"> = splash
      .mul(upward.mul(applied))
      .add(fallX.yxz.mul(abs(facing.x).mul(applied)))
      .add(fallZ.zxy.mul(abs(facing.z).mul(applied)));
    const bent: Node<"vec3"> = cameraViewMatrix.mul(vec4(water.x, water.y, water.z.negate(), 0)).xyz;

    return vec4(normalize(normal.add(bent)), wetness.mul(dot(albedo, vec3(0.33))));
  })();
}

/**
 * `rain_apply_normal`: the patched normal written back where anything was drawn.
 *
 * @param patched - What the patch wrote.
 * @param depth - The G-buffer's depth.
 * @returns The fragment for the normal target.
 */
export function toWetNormalFragment(patched: Texture, depth: Texture): Node<"vec4"> {
  return Fn(() => {
    If(texture(depth, screenUV).x.lessThanEqual(0), () => {
      Discard();
    });

    return vec4(encodeOctahedral(texture(patched, screenUV).xyz), 0, 1);
  })();
}

/**
 * `rain_apply_gloss`: what the albedo is multiplied by, darker the wetter, and the gloss the wetness adds, which the
 * blend adds. The extended engine brightens by half the rain what it wets least.
 *
 * @param input - What the patch wrote, the G-buffer's depth, the wet surfaces and the engine.
 * @returns The fragment for the albedo target.
 */
export function toWetGlossFragment(input: IWetGlossInput): Node<"vec4"> {
  return Fn(() => {
    const { patched, depth, wet, engine } = input;

    If(texture(depth, screenUV).x.lessThanEqual(0), () => {
      Discard();
    });

    const gloss: Node<"float"> = texture(patched, screenUV).w;
    const darker: Node<"float"> = sqrt(gloss).oneMinus();
    const intensity: Node<"float"> = select(
      isExtendedEngine(engine),
      darker.add(wet.density.mul(0.5)),
      max(darker, 0.5)
    );

    return vec4(intensity, intensity, intensity, gloss.mul(toEngineValue(engine, WET_GLOSS)));
  })();
}
