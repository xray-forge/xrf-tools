import { Nullable } from "@xrf/types";
import { Color, LinearSRGBColorSpace, NodeMaterial, QuadMesh, Texture } from "three/webgpu";

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

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly backdrop: Color = new Color();
  private readonly targets: RendererTargets;
  private readonly uniforms: RendererUniforms;
  private material: NodeMaterial;
  private ambientOcclusion: Nullable<Texture> = null;

  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.uniforms = uniforms;
    this.material = this.createMaterial();
  }

  /**
   * @param ambientOcclusion - The screen's occlusion combine multiplies the hemisphere and ambient by from now on, or none.
   */
  public setAmbientOcclusion(ambientOcclusion: Nullable<Texture>): void {
    if (ambientOcclusion === this.ambientOcclusion) {
      return;
    }

    this.ambientOcclusion = ambientOcclusion;
    this.material.dispose();
    this.material = this.createMaterial();
  }

  public render({ renderer, targets, settings }: IRendererFrame): void {
    // The hex is bytes the page shows, so it reaches the canvas as written rather than decoded from srgb.
    this.backdrop.setHex(settings.backdrop ?? 0, LinearSRGBColorSpace);
    renderer.setClearColor(this.backdrop, settings.backdrop === null ? 0 : 1);
    renderer.setRenderTarget(targets.scene);
    renderer.clear(true, false, false);
    this.quad.material = this.material;
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
  }

  private createMaterial(): NodeMaterial {
    return createQuadMaterial(
      toCombinePassFragment(this.targets, this.targets.light.texture, this.uniforms, this.ambientOcclusion)
    );
  }
}
