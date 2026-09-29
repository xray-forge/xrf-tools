import { TextureNode } from "three/webgpu";

/**
 * A sampler that is its own binding, whatever it holds. Three binds the samplers of one texture once, by its uuid
 * (`TextureNode.getUniformHash`): two slots built holding the same placeholder, or the same texture, would share one
 * binding and both sample the first. Its clones - sampled, at a level - share its binding.
 */
export class SlotTextureNode extends TextureNode {
  public override getUniformHash(): string {
    return `slot:${this.getBase().id}`;
  }
}
