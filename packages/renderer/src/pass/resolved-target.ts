import { DepthTexture, FloatType, RenderTarget, WebGPURenderer } from "three/webgpu";

import { initPreservedDepthTarget } from "#/internals/preserved-depth-target";
import { createColourTarget, IColourAttachment } from "#/pass/colour-target";

/**
 * A frame resolved at the output's size: its colour, and a depth of its own the resolve writes for the helpers drawn
 * over it to test against. The resolve draws through writers, which put colours of their own beside it.
 */
export class ResolvedTarget {
  /** The resolved frame and its depth, which the helpers draw into and present reads. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });

  private readonly others: Array<RenderTarget> = [];

  /**
   * @param name - What the device labels its colour and depth.
   */
  public constructor(name: string) {
    this.output.texture.name = name;
    this.output.depthTexture = new DepthTexture(1, 1, FloatType);
    this.output.depthTexture.name = `${name}-depth`;
  }

  /**
   * @param colours - The colours the resolve writes before the output's, in its attachments' order.
   * @returns A target the resolve draws into: those colours, then the output's, and the depth.
   */
  public createWriter(colours: ReadonlyArray<IColourAttachment>): RenderTarget {
    const target: RenderTarget = createColourTarget([...colours, { name: this.output.texture.name }], true);

    target.textures[colours.length].dispose();
    target.textures[colours.length] = this.output.texture;
    target.depthTexture = this.output.depthTexture;
    this.others.push(target);

    return target;
  }

  /**
   * Sizes the output and everything sharing its depth and allocates them on the renderer: the output first, since
   * three sizes a shared depth by the target it was allocated with, and none clearing the depth on its first draw.
   *
   * @param renderer - The renderer drawing them.
   * @param width - The output's width.
   * @param height - And its height.
   */
  public resize(renderer: WebGPURenderer, width: number, height: number): void {
    this.output.setSize(width, height);
    this.others.forEach((target: RenderTarget) => target.setSize(width, height));
    initPreservedDepthTarget(renderer, this.output);
    this.others.forEach((target: RenderTarget) => initPreservedDepthTarget(renderer, target));
  }

  /** Lets the output, its depth, and every writer go, with every colour they write. */
  public dispose(): void {
    this.others.forEach((target: RenderTarget) => target.dispose());
    this.output.dispose();
    this.output.texture.dispose();
    this.output.depthTexture?.dispose();
  }
}
