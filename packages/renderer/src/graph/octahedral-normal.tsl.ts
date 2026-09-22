import { abs, float, Fn, normalize, saturate, select, vec2, vec3 } from "three/tsl";
import { Node } from "three/webgpu";

/** One or minus one by sign, counting zero as positive, so an axis-aligned normal still folds. */
const signNotZero = Fn(([value]: [Node<"float">]) => select(value.greaterThanEqual(0), float(1), float(-1)));

/**
 * Folds a unit normal onto the octahedron and flattens it into two numbers in `[-1, 1]`.
 */
export const encodeOctahedral = Fn(([normal]: [Node<"vec3">]) => {
  const n = normal.div(abs(normal.x).add(abs(normal.y)).add(abs(normal.z)));
  const folded = vec2(float(1).sub(abs(n.y)).mul(signNotZero(n.x)), float(1).sub(abs(n.x)).mul(signNotZero(n.y)));

  return select(n.z.greaterThanEqual(0), n.xy, folded);
});

/**
 * Unfolds what `encodeOctahedral` wrote back into a unit normal.
 */
export const decodeOctahedral = Fn(([encoded]: [Node<"vec2">]) => {
  const n = vec3(encoded.x, encoded.y, float(1).sub(abs(encoded.x)).sub(abs(encoded.y)));
  const t = saturate(n.z.negate());

  return normalize(
    vec3(
      n.x.add(select(n.x.greaterThanEqual(0), t.negate(), t)),
      n.y.add(select(n.y.greaterThanEqual(0), t.negate(), t)),
      n.z
    )
  );
});
