import { CubeTextureNode } from "three/webgpu";

/**
 * A cube sampler that is its own binding, whatever it holds, as `SlotTextureNode` is: the two skies of a blend built
 * holding one placeholder would otherwise both sample the first.
 */
export class SlotCubeTextureNode extends CubeTextureNode {
  public override getUniformHash(): string {
    return `slot:${this.getBase().id}`;
  }
}
