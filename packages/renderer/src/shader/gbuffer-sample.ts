import { Node } from "three/webgpu";

import { IBaseShadingPoint } from "#/shader/base-shading-point";

/**
 * One pixel of the G-buffer, decoded.
 */
export interface IGBufferSample {
  /** Hardware depth: one where nothing was drawn. */
  depth: Node<"float">;
  /** Raw albedo. */
  albedo: Node<"vec3">;
  gloss: Node<"float">;
  /** The baked hemisphere occlusion. */
  hemi: Node<"float">;
  /** The baked sun occlusion. */
  sun: Node<"float">;
  /** The point stored there, its view position rebuilt from depth. */
  point: IBaseShadingPoint;
}
