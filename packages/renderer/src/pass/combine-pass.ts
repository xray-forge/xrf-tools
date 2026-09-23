import { Color, LinearSRGBColorSpace, NodeMaterial, QuadMesh } from "three/webgpu";

import { toCombinePassFragment } from "#/pass/combine-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `combine_1` and the tonemap of `combine_2`: the hemisphere model, the lights, fog, and the engine's Reinhard curve.
 */
export class CombinePass implements IRendererPass {
  public readonly name: string = "combine";

  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;
  private readonly backdrop: Color = new Color();

  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.material = createQuadMaterial(toCombinePassFragment(targets, targets.light.texture, uniforms));
    this.quad = new QuadMesh(this.material);
  }

  public render({ renderer, targets, settings }: IRendererFrame): void {
    // The hex is bytes the page shows, so it reaches the canvas as written rather than decoded from srgb.
    this.backdrop.setHex(settings.backdrop ?? 0, LinearSRGBColorSpace);
    renderer.setClearColor(this.backdrop, settings.backdrop === null ? 0 : 1);
    renderer.setRenderTarget(targets.scene);
    renderer.clear(true, false, false);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
  }
}
