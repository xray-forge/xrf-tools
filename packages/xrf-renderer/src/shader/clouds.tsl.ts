import { float, max, mix, normalize, pow, saturate, vec2, vec3 } from "three/tsl";
import { Node } from "three/webgpu";

import { toBoxDirection } from "#/shader/sky.tsl";
import { CloudUniforms } from "#/uniforms/cloud-uniforms";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/** `mScale.scale(10, 0.4f, 10)`: the dome `RenderClouds` draws, wide and flat around the camera. */
const DOME_WIDTH: number = 10;
const DOME_HEIGHT: number = 0.4;

/** `CLOUD_TILE0`, `CLOUD_SPEED0`, `CLOUD_TILE1` and `CLOUD_SPEED1` (`shared/cloudconfig.h`). */
const TILE_0: number = 0.7;
const SPEED_0: number = 2 * 0.05;
const TILE_1: number = 2.8;
const SPEED_1: number = 2 * 0.025;

/** How sharply the clouds fade to the horizon, `pow(v.p.y, 25)` in `clouds.vs`. */
const FADE: number = 25;

/**
 * A direction through the byte `RenderClouds` packs it into, as `v.dir * 2 - 1` reads it back.
 *
 * @param value - A component in minus one to one.
 * @returns What the vertex shader reads of it.
 */
function toPacked(value: number): number {
  return (Math.floor((value * 0.5 + 0.5) * 255) / 255) * 2 - 1;
}

/** The two layers' drift, `setHP(PI_DIV_4, 0)` and `setHP(PI_DIV_4 + PI_DIV_8, 0)` in `x` and `z`. */
const WIND_0: readonly [number, number] = [toPacked(-Math.sin(Math.PI / 4)), toPacked(Math.cos(Math.PI / 4))];
const WIND_1: readonly [number, number] = [
  toPacked(-Math.sin(Math.PI / 4 + Math.PI / 8)),
  toPacked(Math.cos(Math.PI / 4 + Math.PI / 8)),
];

/** What the clouds are laid over the sky by. */
interface ISkyWithCloudsUniforms {
  sky: SkyUniforms;
  clouds: CloudUniforms;
  /** What the tonemap multiplies by. */
  scale: Node<"float">;
}

/**
 * The clouds as `RenderClouds` lays them over the sky before combine (`clouds.vs`, `clouds.ps`): the dome met along the
 * view, its point in the dome's own axes tiling two scrolling layers of both keyframes' textures, blended by the
 * keyframes' weight, times the clouds' colour, faded to the horizon and blended over by their cover.
 *
 * @param direction - A direction in renderer space.
 * @param below - The sky behind the clouds along it.
 * @param uniforms - The sky's and the clouds' uniforms, and what the tonemap multiplies by.
 * @returns The sky with the clouds over it.
 */
export function toSkyWithClouds(
  direction: Node<"vec3">,
  below: Node<"vec3">,
  uniforms: ISkyWithCloudsUniforms
): Node<"vec3"> {
  const { sky, clouds, scale } = uniforms;
  const box: Node<"vec3"> = toBoxDirection(direction, clouds.rotation);
  // The unit hemisphere point the dome's scale puts along the direction: `v.p`.
  const dome: Node<"vec3"> = normalize(vec3(box.x.div(DOME_WIDTH), box.y.div(DOME_HEIGHT), box.z.div(DOME_WIDTH)));
  // `timers.z`, the time a tenth as fast.
  const drift: Node<"float"> = clouds.time.mul(0.1);
  const first: Node<"vec2"> = dome.xz.mul(TILE_0).add(vec2(WIND_0[0], WIND_0[1]).mul(drift.mul(SPEED_0)));
  const second: Node<"vec2"> = dome.xz.mul(TILE_1).add(vec2(WIND_1[0], WIND_1[1]).mul(drift.mul(SPEED_1)));
  const [from, to] = clouds.textures;
  const texel: Node<"vec4"> = mix(
    from.sample(first).add(from.sample(second)),
    to.sample(first).add(to.sample(second)),
    sky.blend
  );
  const color: Node<"vec4"> = clouds.color.mul(texel);
  const cover: Node<"float"> = saturate(color.w.mul(pow(max(dome.y, float(0)), float(FADE)))).mul(clouds.drawn);

  return mix(below, color.xyz.mul(scale), cover);
}
