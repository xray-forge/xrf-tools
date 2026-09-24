import { Node } from "three/webgpu";

/**
 * The nodes a surface's material is built from, as the pass drawing it shades.
 */
export interface ISurfaceShader {
  /** The whole fragment output, for a pass writing several targets. */
  fragmentNode?: Node;
  /** The colour and alpha, for a pass writing one. */
  colorNode?: Node<"vec4">;
  /** The alpha a texel must exceed to be drawn at all. */
  alphaTestNode?: Node<"float">;
  /** Where a vertex stands in view space, for a surface that places its own vertices. */
  positionViewNode?: Node<"vec3">;
}
