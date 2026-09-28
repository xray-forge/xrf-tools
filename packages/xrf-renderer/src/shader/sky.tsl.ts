import { abs, cos, float, If, max, mix, select, sin, smoothstep, vec3 } from "three/tsl";
import { Node } from "three/webgpu";

import { SkyUniforms } from "#/uniforms/sky-uniforms";

/** `sky2.vs` and `sky2.ps`: the colour doubled on the tonemap's scale, then the blended cubes a third of it. */
const SKY_FACTOR: number = 2 * 0.33;

/** Where `hbox_verts` puts the ring the box's sides fold at, just under the horizon. */
const BOX_HORIZON: number = -0.01;

/** How far below the bottom face the box's lower half samples, so all of it reads the cube's bottom rim. */
const BOX_BOTTOM: number = -1.01;

/** How high above the fold the haze is still averaged, fading back to single taps by then. */
const HAZE_TOP: number = 0.04;

/** Taps the haze is averaged over, and the turn between two, two texels of a 512 face: a box over some twenty-four. */
const HAZE_TAPS: number = 13;
const HAZE_TAP_ANGLE: number = (0.35 * Math.PI) / 180;

/**
 * A renderer-space direction in the sky box's own axes: the engine's `z` negated back, then turned back by the
 * `rotateY(sky_rotation)` the box is drawn with.
 */
function toBoxDirection(direction: Node<"vec3">, rotation: Node<"float">): Node<"vec3"> {
  const x: Node<"float"> = direction.x;
  const z: Node<"float"> = direction.z.negate();
  const c: Node<"float"> = cos(rotation);
  const s: Node<"float"> = sin(rotation);

  return vec3(x.mul(c).sub(z.mul(s)), direction.y, x.mul(s).add(z.mul(c)));
}

/**
 * The cube coordinate `hbox_verts` gives a direction: the top face as it is, each side's whole height folded into
 * the sky above the horizon, and everything below it reading the bottom rim, which is the haze a cube is painted with.
 */
function toBoxLookup(box: Node<"vec3">): Node<"vec3"> {
  const side: Node<"float"> = max(abs(box.x), abs(box.z));
  const height: Node<"float"> = box.y.div(side);
  // Each side's two bands, straight across in the face's plane as its vertices interpolate: `half` to the top, the
  // bottom to `half`.
  const upper: Node<"float"> = float(-1).add(height.sub(BOX_HORIZON).mul(2 / (1 - BOX_HORIZON)));
  const lower: Node<"float"> = float(BOX_BOTTOM).add(height.add(1).mul((-1 - BOX_BOTTOM) / (1 + BOX_HORIZON)));
  const onSide: Node<"vec3"> = vec3(
    box.x.div(side),
    select(height.greaterThanEqual(BOX_HORIZON), upper, lower),
    box.z.div(side)
  );
  const onBottom: Node<"vec3"> = vec3(box.x.div(box.y.negate()), BOX_BOTTOM, box.z.div(box.y.negate()));

  return select(box.y.greaterThan(side), box, select(box.y.negate().greaterThan(side), onBottom, onSide));
}

/** An engine cube coordinate as three samples it: its own flip of `x` on cube lookups undone. */
function toCubeLookup(engine: Node<"vec3">): Node<"vec3"> {
  return vec3(engine.x.negate(), engine.y, engine.z);
}

/** Both skies at a cube coordinate, blended as `L_ambient.w` blends them, at their top level: a sky ships no mips. */
function toBlendedCubes(lookup: Node<"vec3">, sky: SkyUniforms): Node<"vec3"> {
  return mix(
    sky.cubes[0].sample(lookup).level(float(0)).xyz,
    sky.cubes[1].sample(lookup).level(float(0)).xyz,
    sky.blend
  );
}

/**
 * @param direction - A direction in renderer space.
 * @param sky - The sky's uniforms.
 * @returns The two skies along it, as a surface reflecting them reads the cube: straight, with no box between.
 */
export function toSkyCubes(direction: Node<"vec3">, sky: SkyUniforms): Node<"vec3"> {
  return toBlendedCubes(toCubeLookup(toBoxDirection(direction, sky.rotation)), sky);
}

/**
 * The sky as `RenderSky` draws it: the cubes through the half box, times `sky_color`, on the exposure's scale, written
 * without the tonemap's curve as `sky2.ps` writes it.
 *
 * @param direction - A direction in renderer space.
 * @param sky - The sky's uniforms.
 * @param scale - What the tonemap multiplies by.
 * @returns The sky's colour as the frame shows it.
 */
export function toSkyColor(direction: Node<"vec3">, sky: SkyUniforms, scale: Node<"float">): Node<"vec3"> {
  const box: Node<"vec3"> = toBoxDirection(direction, sky.rotation);
  const height: Node<"float"> = box.y.div(max(abs(box.x), abs(box.z)));
  const color: Node<"vec3"> = toBlendedCubes(toCubeLookup(toBoxLookup(box)), sky).toVar();

  // Below the fold every pixel of a column reads one texel of the bottom rim, which draws its block noise as streaks
  // the height of the band; averaged around the compass there, it keeps the haze and loses the texels.
  If(height.lessThan(HAZE_TOP), () => {
    let sum: Node<"vec3"> = vec3(0);

    for (let tap: number = 0; tap < HAZE_TAPS; tap += 1) {
      const angle: number = (tap - (HAZE_TAPS - 1) / 2) * HAZE_TAP_ANGLE;
      const turned: Node<"vec3"> = vec3(
        box.x.mul(Math.cos(angle)).sub(box.z.mul(Math.sin(angle))),
        box.y,
        box.x.mul(Math.sin(angle)).add(box.z.mul(Math.cos(angle)))
      );

      sum = sum.add(toBlendedCubes(toCubeLookup(toBoxLookup(turned)), sky));
    }

    color.assign(mix(sum.div(HAZE_TAPS), color, smoothstep(BOX_HORIZON, HAZE_TOP, height)));
  });

  return color.mul(sky.color).mul(scale.mul(SKY_FACTOR));
}
