import { Fn } from "three/tsl";
import { Node } from "three/webgpu";

/**
 * The tangent space normal `sload_i` reconstructs from a texel of the pair: `Nu.wzy + (NuE.xyz - 1)`, unnormalised.
 */
export const decodeBumpNormal = Fn(([bump, companion]: [Node<"vec4">, Node<"vec4">]) =>
  bump.wzy.add(companion.xyz.sub(1))
);

/** `S.gloss = Nu.x * Nu.x`. */
export const decodeBumpGloss = Fn(([bump]: [Node<"vec4">]) => bump.x.mul(bump.x));

/** Height as the generator writes it and parallax reads it: the companion's alpha. */
export const decodeBumpHeight = Fn(([companion]: [Node<"vec4">]) => companion.w);
