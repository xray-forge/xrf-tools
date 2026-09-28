import { uniform } from "three/tsl";
import { UniformNode } from "three/webgpu";

/**
 * How far one build's ring reaches: the planting's reach, or less where the build holds fewer cells, so a build still
 * planting while a larger one compiles never reads or writes past its own ring.
 */
export class GrassRingUniforms {
  /** Slots the ring reaches each way from the camera's. */
  public readonly reach: UniformNode<"float", number> = uniform(0);

  /** Cells the ring holds at its reach. */
  public get cells(): number {
    const line: number = this.reach.value * 2 + 1;

    return line * line;
  }

  /**
   * @param reach - Slots the planting reaches each way.
   * @param bands - Rings of cells the build holds around the camera's, its own among them.
   */
  public fit(reach: number, bands: number): void {
    this.reach.value = Math.min(reach, bands - 1);
  }
}
