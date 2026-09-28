import { Node } from "three/webgpu";

/** The four texels a bilinear read of a point takes, and each one's weight. */
export interface IBilinearFootprint {
  /** The first texel, held as floats. */
  base: Node<"vec2">;
  corners: ReadonlyArray<{ offset: readonly [number, number]; weight: Node<"float"> }>;
}
